//! Defines our main API (using the Builder Pattern) & related internal model
//! to allow async "dynamic" algorithms analysis.
//! 
//! By "dynamic algorithms" we mean "algorithms that do alter the amount of data they operate on"
//! -- such as algorithms to insert & delete data.
//! This is in opposition to [super::regular_async_builder].

// TODO