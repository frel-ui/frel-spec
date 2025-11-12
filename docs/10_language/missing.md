## Missing stuff to think about

Priorities are assigned by Claude, the comments next to them are added by me.

Derived/Filtered Collections 🔴 High Priority - not really high, for MVP I can live without it
Selection State 🔴 High Priority - simply part of the data, not something to worry about at data model level
Transient UI State 🔴 High Priority - these are **not ephemeral**, they are part of the application state
Pagination/Infinite Scroll 🟡 Medium Priority - MVP can live without it
Many-to-Many Relationships 🟡 Medium Priority - List of Schemes with "ref" - "ref" or "ref" - "string"
Optimistic Updates 🟡 Medium Priority - I would simply say "no optimistic updates"
Cross-Scheme Validation 🟡 Medium Priority - MVP can live wirthout it
Polymorphic/Union Types 🟡 Medium Priority - No polymorphic types, "include" statements for composition (might add to schemes as well)
Aggregation/Grouping 🟢 Lower Priority - backend concern
Undo/Redo 🟢 Lower Priority - actual application concern
User Preferences/Settings 🟢 Lower Priority - just schemes
Search/Sort/Filter Abstractions 🟢 Lower Priority - library level concern