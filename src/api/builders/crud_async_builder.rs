//! Defines our main API (using the Builder Pattern) & related internal model
//! to allow async CRUD algorithms analysis.
//! 
//! CRUD algorithms -- to create, read, update and delete data -- must be analysed with
//! different strategies: "Regular Algorithm Analysis" for the read and update operations
//! and "Dynamic Algorithm Analysis" for create and delete operations.
//! 
//! Please refer to [super::dynamic_async_builder] and [super::crud_async_builder] for more info.

// TODO