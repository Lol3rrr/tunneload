initSidebarItems({"enum":[["AsStringError","Errors that can occur when serializing a [`AsString`] wrapper."],["LockError","Errors that can occur while serializing a [`Lock`] wrapper"],["UnixTimestampError","Errors that can occur when serializing a [`UnixTimestamp`] wrapper."]],"struct":[["AsBox","A wrapper that serializes a field into a box."],["AsOwned","A wrapper that serializes a `Cow` as if it were owned."],["AsString","A wrapper that attempts to convert a type to and from UTF-8."],["AsVec","A wrapper that serializes associative containers as a `Vec` of key-value pairs."],["Atomic","A wrapper that archives an atomic with an underlying atomic."],["CopyOptimize","A wrapper that provides specialized, performant implementations of serialization and deserialization."],["Immutable","A wrapper to make a type immutable."],["Inline","A wrapper that serializes a reference inline."],["Lock","A wrapper that locks a lock and serializes the value immutably."],["Niche","A wrapper that niches some type combinations."],["RefAsBox","A wrapper that serializes a reference as if it were boxed."],["UnixTimestamp","A wrapper that converts a `SystemTime` to a `Duration` since `UNIX_EPOCH`."],["With","A transparent wrapper for archived fields."]],"trait":[["ArchiveWith","A variant of [`Archive`] that works with [`With`] wrappers."],["DeserializeWith","A variant of `Deserialize` that works with `With` wrappers."],["SerializeWith","A variant of `Serialize` that works with `With` wrappers."]],"type":[["Boxed","A wrapper that serializes a reference as if it were boxed."]]});