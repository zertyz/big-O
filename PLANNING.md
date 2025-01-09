# Evolutions & History for this project

Here you'll find the *backlogs*, *being-done*, *completed* and *bug reports* for this project.

Issues contain a *prefix* letter and a sequence number, possibly followed by a date and a context-full description.



## Prefixes

- (b) bug fix for broken functionalities
- (f) new functional requisite
- (n) new non-functional requisite
- (r) internal re-engineering / refactor / improvement, to increase speed and/or enable further progress to be done cheaper



# Being-Done

**(n1)** 2025-01-09: Redo our API (for v1.0), using the builder pattern, to match the following sketch:
```
big_o::analyse_regular_async_algorithm()
  .with_warmup(|| async {...})
  .with_max_reattempts_per_pass(2)
  .with_reset_fn(|| async {...})
  .first_pass(n_elements, |n_elements| async {...} -> data)
  .first_pass_assertions(|data| async {...})
  .second_pass(n_elements, |n_elements| async {...} -> data)
  .second_pass_assertions(|data| async {...})
  .with_time_measurements(BigOThings::On)
  .with_space_measurements(BigOThings::O1)
  .with_auxiliary_space_measurements(BigOThings::On)
  .add_custom_measurement("Δconn", BigOThing::O1, "total connections opened", ValueRepresentation::Unit, |data| ... -> val)
  .add_custom_measurement_with_averages("Δcalls", BigOThing::O1, "total external service calls made", ValueRepresentation::Scientific, |data| ... -> val)
```
Note both the builder and the runner might fail due to sanity check violations in addition to the provided assertions:
a) n is the same for the 1st and 2nd passes -- or even smaller on the 2nd pass... or not "big enough"
b) memory measurements are inconsistent between re-attempts (when the difference is "considerably large")
c) the memory didn't return to the same value after everything was freed -- either a memory leak, caching or concurrent tests are running


# Backlog

**(f2)** 2025-01-09: After **(n1)**, redo our reports to match the following sketch:
```
Running 'Quicksort a reversed vec' algorithm:
  Resetting: 3406857µs/+768.00MiB; Pass 1: 658484µs/76.29MiB; Pass 2: 1315255µs/152.59MiB
  (re-attempt)                                                Pass 2: 1315255µs/152.59MiB
  (re-attempt)                     Pass 1: 658484µs/76.29MiB

'Quicksort a reversed vec' regular-algorithm measurements:
Pass 1: n = 40000000                 Pass 2: n = 80000000                 Analysis         Info
  Δt:     658484µs   avg: 0.016µs      Δt:     1315255µs  avg: 0.016µs      ✓ O(n)           time taken
  Δs:     76.29MiB   avg: 2b           Δs:     152.59MiB  avg: 2b           ✗ O(n) ⪢ O(1)    used / unfreed space after
  max s:  228.88MiB  avg: 6b           max s:  228.88MiB  avg: 6b           ✓ O(1) ⪡ O(n)    peak auxiliary space used
  Δconn:  1                            Δconn: 1                             ✓ O(1)           total connections opened
  Δcalls: 4e7       ; avg: 1           Δcalls: 8e7      ; calls⁻: 1         ✓ O(n)           total external service calls made

Algorithm Analysis checks failed:
  "used / unfreed space" required an O(1) complexity but was measured as O(n)
  <<if failed due to time measurements and measurements in re-attempts were not consistent, suggest either increasing the reattempts limit, making sure the machine is idle and running no other tests or decreasing the precision>>

All Algorithm Analysis checks succeeded!

Notes:
  Re-attempts due to timing skews: Pass 1 (2 times); Pass 2 (2 times) with x seconds lost
    --> maybe the machine is not idle enough or you may consider using the warmup mechanism
  "used / unfreed space after" performed much better than expected
    --> things improved and maybe it is time to raise the bar for this property

```
Things to notice:
1) This story is only for creating the model able to represent the above report
2) It should provide a rudimentary output -- not looking as good as the sketch above
3) **(f3)** is to wrap this up and make it as beautiful as the sketch above

**(f3)** 2025-01-09: The follow-up of **(f2)**, make the report easy, pleasent to look and beautiful:
1) See the sketch in the referred story. This is how the final report should look
2) Notice the report have several "tabs" or "columns". They should be honored and the crate `prettytable`, v0.10, helps.
3) Take the following as a starter (it was encoded with `gzip -9 | base64 -w 0`; decode it with `base64 -d | gunzip`): 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 



# Done