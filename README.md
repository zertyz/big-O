# the *big-O* crate

big-O is a framework for running *space* and *time* Algorithm Complexity Analysis on numerical, data manipulation,
CRUD containers, database models & queries, as well as business logics that depend on such algorithms -- measuring
how memory allocations and function call times increase in relation to the amount of data they are applied to and,
possibly more important, up to how many threads may be recruited in order to achieve the best possible throughput
or the best balance between troughtput and latency.

The purpose of this crate is to allow you to foresee your application's performance and be prepared for when those 
big data scenarios come to be. It is meant to work as a *development tool*, alongside with *tests* & *benchmarks*.

When used in *benchmarks*, together with 'Criterion.rs', 'big-O' reports run-time complexity analysis on speeds and memory
*measurements*, so you can analyze improvements in a measurable and repeatable way and obtain the *optimum* number of
*parallel processors* for your algorithm.

When used in *tests*, it enforces a *minimum big-O notation complexity*, denying regressions in space or time complexity
while also assuring changes and optimizations to your algorithm remain *free of concurrency issues* -- due to the nature
of the multi-threaded algorithm analysis, it is also able to assure the reentrancy & concurrency correctness of your code
-- even when Rust makes guarantees regarding data races, interactions with external services (like databases) may still
raise concurrency issues -- which this crate is able to test.

In short, here is how some performance measurement tools might fit in your development stack:
  - FlameGraphs: used to spot bottlenecks -- may be used sporadically;
  - 'Criterion.rs': measure function call times on predefined inputs -- maybe on slow functions you want to watch closely, 
    spotted with a flamegraph. You want this to run on every build and to pass certain performance requirements;
  - 'big-O': measure & assure a minimum acceptable Algorithm Complexity (in the big-O notation) for functions that depend on 
    the ammount of data they work on, while also testing their concurrency (in case of CRUD set of functions). You want this 
    to be called by some test cases as well as by some benchmarks.

## Basic Usage

### Algorithm Analysis tests

### Concurrency tests

### benchmarks

## Advanced Usage

#### repeating failed tests