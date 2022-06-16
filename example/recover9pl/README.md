# recover9pl

A proxy server for 9p2000.l that tracks attached fids
and is able to re-establish the attach points on network
failure. This proxy means you don't need to re-mount your filesystem on disconnect.

Any requests that happen during a disconnected period will
get EIO as a response to any messages. Any open files will
return an error for all requests until they are closed.

## Important edge cases

### File locks and State

You must pay special attention to state (especially file locks) programs
acquire while doing multiple walks of the attach fid.
A disconnect will release locks and other state acquired since the first walk
and your program may not detect this until it is too late.

The best solution is to write your application such that
it uses 'openat' from a file descriptor that was retrieved from an initial Twalk
message. All file access should go via this file descriptor and its descendents.
In this case, if a reconnection happens your program will gracefully
fail with an IO error for all future file access until it is restarted.

## Implementation notes

### Clunk and Remove

The 9p protocol says clients must respect Tclunk and Tremove
even on error. We take advantage of this as reconnected
will just respond with BADF when clunking a file it doesn't know about, the client then forgets the fid as normal.

### Walks

The proxy doesn't currently track and reestablish an empty
Twalk of an attach fid. This could be added in the future but
did not seem necessary.