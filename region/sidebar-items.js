initSidebarItems({"enum":[["Error","A collection of possible errors."]],"fn":[["lock","Locks one or more memory regions to RAM."],["protect","Changes the memory protection of one or more pages."],["protect_with_handle","Changes the memory protection of one or more pages temporarily."],["query","Queries the OS with an address, returning the region it resides within."],["query_range","Queries the OS with a range, returning the regions it contains."],["unlock","Unlocks one or more memory regions from RAM."]],"mod":[["page","Page related functions."]],"struct":[["LockGuard","An RAII implementation of a “scoped lock”. When this structure is dropped (falls out of scope), the virtual lock will be unlocked."],["ProtectGuard","An RAII implementation of “scoped protection”. When this structure is dropped (falls out of scope), the memory region protection will be reset."],["Protection","Memory page protection constants."],["Region","A descriptor for a memory region"]],"type":[["Result","The result type used by this library."]]});