# Simple file backup

This tool watches provided source files and automatically copies them to destination folder by adding an index to a filename.

Initial design and implementation comes from a requirement to backup multiple save files when game uses autosave feature or has very limited amount of slots.

WARNING: This way of backing up is neither secure nor perfect.
         In most scenarios there's better solution.

## Initial design choices and limitations

At the moment there's few known design limitations, which maybe will improved in the future.

## Assorted list of design considerations and limitations

Consider the section as an unofficial TODO list. Feel free to make a PR to fix any of them or discuss to gather more information and/or propose a solution ideas.

Files, target file naming and input

* Target file format is fixed: `<original-filename>_<hex counter>.<extension>`.
* UTF-8 paths are used to simplify coding.
  This could be a limitation on Windows.
* In some cases on Windows names with spaces are not really recognized as full filename, probably a CMD script thing.
* Absolute filenames are used internally.
* Only plain files are supported.
* File permission checks for source and target folders are quite lazy and incomplete.
* Input files must exist prior to run the application,
  but it may be ok to delete and restore it while it's running.
* Filesystem notifications are debounced by `100ms` and it's not configurable.
* `std::fs::copy` is used to copy files (filename to filename).
* Move operation (opposite to copy) was considered — like Vim does.
  However, file should be left as it was to avoid collisions and logic of some games.
* Filesystems R/W locks are not used, so it's possible that other process/thread will overwrite a file during copy process.
  Most game engines ignore errors on file write, and an R/W lock may lead to broken save files.
* Only `Modify` event is used to determine when it's needed to copy a file.
  I haven't tested other events.
* File name collisions is a possibility when file types are mixed.
* Time always goes forward.
  If a person changed time on a computer then saved a file, it may not be backed up.

Logging and output

* Logging is dump and simple, console only, not configurable.
  `simplelog` probably is not the best choice, but it works well enough.
* ~~Time is logged in nanoseconds~~.
* ~~Actual event logging time is not displayed~~.
* Time measurements are in nanoseconds.

Internals

* There's few sync gates for a state: provided by `papaya::HashMap` and `AtomicBool` inside `Arc` inside a state structure.
  It's possible that this is an overkill for sync, as it supposed to be updated once in a debounce time.
* While architecture design with global cells is questionable.
  I see no other way to push data to `notify` listener thread.
* File prefix removal requires conversion to UTF-8, which may fail.

Testing

* App is manually tested on Windows and macOS under limited conditions like fast SSDs, relatively small files (less than 50mb in total) and enough memory to read multiple of them at once.
