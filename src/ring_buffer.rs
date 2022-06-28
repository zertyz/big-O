//! Provides a concurrent zero-copy ring-buffer with multiple consumers, with the following characteristics:
//!   1) Zero-cost multiple & independent consumers are allowed -- like consumer-groups in a queue topic: each will see all the events;
//!   2) Each consumer is concurrent and may still be shared by several threads;
//!   3) Each consumer holds their own state (their 'head' pointer), therefore access should be done through a special structure [RingBufferConsumer]
//!   4) Due to (1), any buffer overflows happens silently in the producer, when enqueueing -- overflows are only detectable by the consumers.
//!      Please see more on [RingBufferConsumer] docs;
use std::sync::atomic::{AtomicU32, Ordering};
use std::mem::MaybeUninit;
use std::io::ErrorKind;
use std::fmt::{Display, Formatter};


/// Represents a concurrent, zero-copy, zero-cost multiple-consumers Ringer buffer.\
/// Create a new ring buffer with 1024 [u32] slots with:
/// ```
///   let ring_buffer = crate::big_o::ring_buffer::RingBuffer::<u32, 1024>::new();
/// ```
/// Note: for optimization purposes, make the ring buffer size a power of 2 -- so that the modulus operation gets optimized to a bit shift instruction.\
/// See [self] for more info.
pub struct RingBuffer<Slot, const RING_BUFFER_SIZE: usize> {
    /// if ahead of [published_tail], indicates new slots is being filled in, to soon be published
    reserved_tail: AtomicU32,
    /// once the slot data is set in place, this counter increases to indicate a new element is ready to be consumed
    published_tail: AtomicU32,
    /// the data
    buffer: MaybeUninit<[Slot; RING_BUFFER_SIZE]>,
}

impl<Slot, const RING_BUFFER_SIZE: usize> RingBuffer<Slot, RING_BUFFER_SIZE> {

    pub const fn new() -> Self {
        Self {
            reserved_tail: AtomicU32::new(0),
            published_tail: AtomicU32::new(0),
            buffer: MaybeUninit::uninit(),
        }
    }

    /// creates a consumer able to consume elements produced after this call
    pub fn consumer(&self) -> RingBufferConsumer<'_, Slot, RING_BUFFER_SIZE> {
        RingBufferConsumer {
            head: AtomicU32::new(self.published_tail.load(Ordering::Relaxed)),
            ring_buffer: &self,
        }
    }

    /// concurrently adds to the ring-buffer, without verifying if this will cause a buffer overflow on any of the consumers
    pub fn enqueue(&self, element: Slot) {

        // reserve the slot
        let reserved_tail = self.reserved_tail.fetch_add(1, Ordering::Relaxed);
        // set the reserved slot contents
        let mutable_buffer = unsafe {
            let const_ptr = self.buffer.as_ptr();
            let mut_ptr = const_ptr as *mut [Slot; RING_BUFFER_SIZE];
            &mut *mut_ptr
        };
        mutable_buffer[reserved_tail as usize % RING_BUFFER_SIZE] = element;

        // publish the new element for consumption
        loop {
            match self.published_tail.compare_exchange_weak(reserved_tail, reserved_tail+1, Ordering::Release, Ordering::Relaxed) {
                Ok(_) => return,
                Err(reloaded_val) => if reloaded_val > reserved_tail {
                    panic!("BUG: Infinite loop detected in Ring-Buffer. Please fix.");
                },
            }
        }
    }

    pub fn get_buffer_size(&self) -> usize {
        RING_BUFFER_SIZE
    }

}


/// Provides a ring-buffer consumer, to be created with:
/// ```
///    let ring_buffer = crate::big_o::ring_buffer::RingBuffer::<u32, 1024>::new();
///    let consumer = ring_buffer.consumer();
/// ```
/// Concurrency note: since, by design, we can have multiple consumers and, for this very reason, the producer is unaware of
/// any consumer state, buffer overflows are not detectable by the producer. Even here, on the consumer, it is only
/// detectable when the buffer overflow happens *before* a call to [dequeue()](RingBufferConsumer::dequeue()) or
/// [peek_all()](RingBufferConsumer::peek_all()) -- if it happens after either of these returns but *before* the
/// references are used, we fall into a **rather silent race condition**.
///
/// There is no way to avoid that possibility (because of the zero-copy & zero-cost multiple consumers characteristics), only to reduce it's effects by:
///   1) Using a big-enough ring-buffer size -- so the buffer won't ever cycle around between *enqueueing* and *using the consumed references*;
///   2) Use the references as fast as possible -- so the *event production speed* won't ever be enough to allow the ring-buffer to cycle around before consumption is done;
///   3) **for the next version:** Design your logic to use something like 'is_reference_valid()', to be called after you're done with the value -- allowing you to check if a race condition happened.
///
/// If these 3 steps are not enough, you might consider using a non-zero-cost multiple consumers ring-buffer, a non-zero-copy one or even a
/// single-consumer ring-buffer. If you know of a way of solving this limitation here, please share.
pub struct RingBufferConsumer<'a, Slot, const RING_BUFFER_SIZE: usize> {
    head: AtomicU32,
    ring_buffer: &'a RingBuffer<Slot, RING_BUFFER_SIZE>,
}
impl<'a, Slot, const RING_BUFFER_SIZE: usize> RingBufferConsumer<'a, Slot, RING_BUFFER_SIZE> {

    /// Zero-copy dequeueing -- returns a reference to the ring-buffer slot containing the dequeued element.
    /// Please note a silent race condition may happen if the ring-buffer's enqueueing operation keeps happening
    /// before this method's caller uses the returned reference. See more on the [RingBufferConsumer] docs.\
    /// Might fail with [RingBufferOverflowError] if the ring buffer had cycled over the element to be dequeued.
    /// Otherwise, returns a reference (if there is some slot to dequeue) or *None* (if there isn't).
    pub fn dequeue(&self) -> Result<Option<&Slot>, RingBufferOverflowError> {
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            let published_tail = self.ring_buffer.published_tail.load(Ordering::Relaxed);
            if head > published_tail {
                head = self.head.load(Ordering::Relaxed);
                continue;
            }
            if head == published_tail {
                return Ok(None);
            }
            match self.head.compare_exchange_weak(head, head + 1, Ordering::Acquire, Ordering::Relaxed) {
                Ok(_) => unsafe {
                    let const_ptr = self.ring_buffer.buffer.as_ptr();
                    let ptr = const_ptr as *const [Slot; RING_BUFFER_SIZE];
                    let array = &*ptr;
                    if self.ring_buffer.reserved_tail.load(Ordering::Relaxed) - head > RING_BUFFER_SIZE as u32 {
                        return Err(RingBufferOverflowError { msg: format!("Ring-Buffer overflow: published_tail={}, head={} -- tail could not be farther from head than the ring buffer size of {}", published_tail, head, RING_BUFFER_SIZE) });
                    }
                    return Ok(Some(&array[head as usize % RING_BUFFER_SIZE]))
                },
                Err(reloaded_head) => head = reloaded_head,
            }
        }
    }

    /// Returns all ring-buffer slot references yet to be [dequeue]ed.\
    /// Although a buffer overflow is detected if it happened before the call to this method,
    /// one might still happen after this method returns and *before* all the references are used
    /// -- so, use this method when you're sure the ring-buffer size & producing speeds are safe.
    ///
    /// The rather wired return type here is to avoid heap allocations: a fixed array of two slices of
    /// the ring buffer are returned -- the second slice is used if the sequence of references cycles
    /// through the buffer. Use this method like the following:
    /// ```
    ///   # fn main() -> std::io::Result<()> {
    ///   let ring_buffer = crate::big_o::ring_buffer::RingBuffer::<u32, 1024>::new();
    ///   let consumer = ring_buffer.consumer();
    ///   // if you don't care for allocating a vector:
    ///   let peeked_references = consumer.peek_all()?.concat();
    ///   // if you require zero-allocations:
    ///   for peeked_chunk in consumer.peek_all()? {
    ///     for peeked_reference in peeked_chunk {
    ///       println!("your_logic_goes_here: {:#?}", *peeked_reference);
    ///     }
    ///   }
    ///   # Ok(())}
    pub fn peek_all(&self) -> Result<[&[Slot];2], RingBufferOverflowError> {
        let head = self.head.load(Ordering::Relaxed);
        let published_tail = self.ring_buffer.published_tail.load(Ordering::Relaxed);
        let head_index           = head as usize % RING_BUFFER_SIZE;
        let published_tail_index = published_tail as usize % RING_BUFFER_SIZE;
        if head == published_tail {
            Ok([&[],&[]])
        } else if published_tail - head > RING_BUFFER_SIZE as u32 {
            Err(RingBufferOverflowError { msg: format!("Ring-Buffer overflow: published_tail={}, head={} -- tail could not be farther from head than the ring buffer size of {}", published_tail, head, RING_BUFFER_SIZE) })
        } else if head_index < published_tail_index {
            unsafe {
                // sorcery to get back an array from a MaybeUninit using only const stable functions (as of Rust 1.55)
                let const_ptr = self.ring_buffer.buffer.as_ptr();
                let ptr = const_ptr as *const [Slot; RING_BUFFER_SIZE];
                let array = &*ptr;
                Ok([&array[head_index .. published_tail_index], &[]])
            }
        } else {
            unsafe {
                // sorcery to get back an array from a MaybeUninit using only const stable functions (as of Rust 1.55)
                let const_ptr = self.ring_buffer.buffer.as_ptr();
                let ptr = const_ptr as *const [Slot; RING_BUFFER_SIZE];
                let array = &*ptr;
                Ok([&array[head_index..RING_BUFFER_SIZE], &array[0..published_tail_index]])
            }
        }
    }

}


/// Indicates the result of a [RingBufferConsumer::dequeue()] or [RingBufferConsumer::peek_all()] operation
/// can no longer be retrieved due to the number of calls to [RingBuffer::enqueue()] causing the ring-buffer
/// to cycle over, overwriting still-unconsumed slot positions in the buffer.\
/// In this case, the consumer instance is no longer valid -- any further operations on it will yield this same error.\
/// A descriptive message is returned in [RingBufferOverflowError::msg].
#[derive(Debug)]
pub struct RingBufferOverflowError {
    /// Contains details on the error
    msg: String,
}
impl Display for RingBufferOverflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RingBufferOverflowError: {}", self.msg)
    }
}
impl std::error::Error for RingBufferOverflowError {}
impl From<RingBufferOverflowError> for std::io::Error {
    fn from(custom_error: RingBufferOverflowError) -> Self {
        std::io::Error::new(ErrorKind::InvalidInput, custom_error)
    }
}


#[cfg(any(test, feature="dox"))]
mod tests {

    //! Unit tests for [ring_buffer](super) module -- using 'serial_test' crate so not to interfere with time measurements from other modules.

    use super::*;

    use serial_test::serial;
    use std::fmt::Debug;


    /// standard use cases assertions for our ring buffer
    #[cfg_attr(not(feature = "dox"), test)]
    fn simple_enqueue_dequeue_use_cases() {
        let ring_buffer = RingBuffer::<i32, 16>::new();
        let consumer = ring_buffer.consumer();

        // dequeue from empty
        match consumer.dequeue() {
            Ok(None) => (),   // test passed
            Ok(Some(existing_element)) => panic!("Something was dequeued when noting should have been: {:?}", existing_element),
            Err(error)   => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
        }

        // enqueue / dequeue a single element
        let expected = 123;
        ring_buffer.enqueue(expected);
        match consumer.dequeue() {
            Ok(None)                         => panic!("No element was dequeued"),
            Ok(Some(existing_element)) => assert_eq!(existing_element, &expected, "Wrong element dequeued"),
            Err(error)   => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
        }

        // circle once through the ring twice, enqueueing / dequeueing a single element at a time
        for i in 0..2*ring_buffer.get_buffer_size() as i32 {
            ring_buffer.enqueue(i);
            match consumer.dequeue() {
                Ok(None)                         => panic!("No element was dequeued"),
                Ok(Some(existing_element)) => assert_eq!(existing_element, &i, "Wrong element dequeued"),
                Err(error)   => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
            }
        }

        // fill in the buffer and then dequeue all elements
        for i in 0..ring_buffer.get_buffer_size() as i32 {
            ring_buffer.enqueue(i);
        }
        for i in 0..ring_buffer.get_buffer_size() as i32 {
            match consumer.dequeue() {
                Ok(None)                         => panic!("No element was dequeued"),
                Ok(Some(existing_element)) => assert_eq!(existing_element, &i, "Wrong element dequeued"),
                Err(error)   => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
            }
        }

        // ensures we end up with an empty ring-buffer
        match consumer.dequeue() {
            Ok(None) => (), // check passed,
            Ok(Some(existing_element)) => panic!("No element should have been left behind, yet {} was dequeued", existing_element),
            Err(error)   => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
        }
    }

    /// [RingBufferConsumer::peek_all()] specification & assertions
    #[cfg_attr(not(feature = "dox"), test)]
    fn peek() -> Result<(), RingBufferOverflowError> {
        let ring_buffer = RingBuffer::<u32, 16>::new();
        let consumer = ring_buffer.consumer();

        let check_name = "empty peek";
        let expected_elements = &[];
        assert_eq!(consumer.peek_all()?.concat(), expected_elements, "{} failed", check_name);

        let check_name = "peek for a single element";
        let expected_elements = &[1];
        ring_buffer.enqueue(1);
        assert_eq!(consumer.peek_all()?.concat(), expected_elements, "{} failed", check_name);

        let check_name = "peek also an additional element";
        let expected_elements = &[1, 2];
        ring_buffer.enqueue(2);
        assert_eq!(consumer.peek_all()?.concat(), expected_elements, "{} failed", check_name);

        let check_name = "peek the whole ring-buffer";
        for e in 3..1+ring_buffer.get_buffer_size() as u32 {
            ring_buffer.enqueue(e);
        }
        let expected_elements: Vec<u32> = (1..1+ring_buffer.get_buffer_size() as u32).into_iter().collect();
        assert_eq!(consumer.peek_all()?.concat(), expected_elements, "{} failed", check_name);

        let check_name = "ring goes round";
        let expected_elements = &[16,17];
        // consume all but the last, leaving only '16' there
        for _ in 1..ring_buffer.get_buffer_size() as u32 {
            consumer.dequeue().unwrap();
        }
        ring_buffer.enqueue(17);
        assert_eq!(consumer.peek_all()?.concat(), expected_elements, "{} failed", check_name);

        let check_name = "EXTRA: demonstration on how to iterate over peeked objects without a vector (or any other) allocation";
        let mut observed_elements = Vec::<u32>::new();
        for peeked_chunk in consumer.peek_all()? {
            for peeked_element in peeked_chunk {
                observed_elements.push(*peeked_element);
            }
        }
        assert_eq!(observed_elements, expected_elements, "{} failed", check_name);

        Ok(())

    }

    /// ensures enqueueing can take place unharmed, but dequeueing & peek_all are prevented (with a meaningful error message) when buffer overflows happens
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial]                 // needed since considerable RAM is used (which may interfere with 'crud_analysis.rs' tests)
    fn buffer_overflowing() {
        let ring_buffer = RingBuffer::<i32, 16>::new();
        let consumer = ring_buffer.consumer();

        // enqueue -- it is impossible to detect buffer overflow since we don't track consumers
        for i in 0..1+ring_buffer.get_buffer_size() as i32 {
            ring_buffer.enqueue(i);
        }

        // peek
        let peeked_chunks = consumer.peek_all();
        assert_buffer_overflow("Peeking", peeked_chunks, "Ring-Buffer overflow: published_tail=17, head=0 -- tail could not be farther from head than the ring buffer size of 16");

        // dequeue
        let element = consumer.dequeue();
        assert_buffer_overflow("Dequeueing", element, "Ring-Buffer overflow: published_tail=17, head=0 -- tail could not be farther from head than the ring buffer size of 16");

        /// asserts the right error was returned
        fn assert_buffer_overflow<E: Debug>(operation: &str, result: Result<E, RingBufferOverflowError>, expected_error_message: &str) {
            if result.is_ok() {
                panic!("{} from an overflowed ring buffer was allowed, when it shouldn't. Returned element was {:?} -- if overflow didn't happen, it would be 0", operation, result);
            } else {
                let observed_error_message = result.unwrap_err().msg;
                assert_eq!(observed_error_message, expected_error_message, "Wrong error message received");
            }
        }

    }

    /// uses varying number of threads for both enqueue / dequeue operations and performs all-in / all-out as well as single-in / single-out tests,
    /// asserting the dequeued element sums are always correct
    #[cfg_attr(not(feature = "dox"), test)]
    #[serial]
    fn concurrency() {

        let ring_buffer = RingBuffer::<u32, 40960>::new();
        let consumer = ring_buffer.consumer();

        // all-in / all-out test -- enqueues everybody and then dequeues everybody
        //////////////////////////////////////////////////////////////////////////
        for threads in 1..16 {

            let start = 0;
            let finish = 40960;

            // all-in (populate)
            multi_threaded_iterate(start, finish, threads, |i| ring_buffer.enqueue(i));

            let expected_sum = (finish - 1) * (finish - start) / 2;
            let observed_sum = AtomicU32::new(0);

            // all-out (consume)
            multi_threaded_iterate(start, finish, threads, |_| match consumer.dequeue() {
                Ok(None) => panic!("Ran out of elements prematurely"),
                Ok(Some(existing_element)) => { observed_sum.fetch_add(*existing_element, Ordering::Relaxed); },
                Err(error) => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
            });
            // check
            assert_eq!(observed_sum.load(Ordering::Relaxed), expected_sum, "Error in all-in / all-out multi-threaded test (with {} threads)", threads);
        }

        // single-in / single-out test -- each thread will enqueue / dequeue a single element at a time
        // (don't set the number of 'threads' too close to the ring-buffer size, or a silent race-condition will take place.
        // See the module docs.)
        ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        let start = 0;
        let finish = 92600;
        let threads = 16;    // might as well be num_cpus::get();

        let expected_sum = (start + (finish-1)) * ( (finish - start) / 2 );
        let expected_callback_calls = finish - start;
        let observed_callback_calls = AtomicU32::new(0);
        let observed_sum = AtomicU32::new(0);

        multi_threaded_iterate(start, finish, threads, |i| {
            observed_callback_calls.fetch_add(1, Ordering::Relaxed);
            // single-in (enqueue)
            ring_buffer.enqueue(i);
            // single-out (dequeue)
            match consumer.dequeue() {
                Ok(Some(existing_element)) => observed_sum.fetch_add(*existing_element, Ordering::Relaxed),
                Ok(None) => panic!("Ran out of elements prematurely"),
                Err(error) => panic!("RingBufferOverflowError while dequeueing : {:?}", error),
            };
        });
        // check
        assert_eq!(observed_callback_calls.load(Ordering::Relaxed), expected_callback_calls, "¿Wrong number of callback calls?");
        assert_eq!(observed_sum.load(Ordering::Relaxed), expected_sum, "Error in single-in / single-out multi-threaded test (with {} threads)", threads);

        /// iterate from 'start' to 'finish', dividing the work among the given number of 'threads', calling 'callback' on each iteration
        fn multi_threaded_iterate(start: u32, finish: u32, threads: u32, callback: impl Fn(u32) -> () + std::marker::Sync) {
            crossbeam::scope(|scope| {
                let cb = &callback;
                let join_handlers: Vec<crossbeam::thread::ScopedJoinHandle<()>> = (start..start+threads).into_iter()
                    .map(|thread_number| scope.spawn(move |_| iterate(thread_number, finish, threads, &cb)))
                    .collect();
                for join_handler in join_handlers {
                    join_handler.join().unwrap();
                }
            }).unwrap();
        }

        /// iterate from 'start' to 'finish' with the given 'step' size and calls 'callback' on each iteration
        fn iterate(start: u32, finish: u32, step: u32, callback: impl Fn(u32) -> () + std::marker::Sync) {
            for i in (start..finish).step_by(step as usize) {
                callback(i);
            }
        }
    }

}