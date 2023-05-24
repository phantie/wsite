<h1>List of imperfections</h1>

I've found some issues that I consider worthy to address in it's source. I provide a list of issues and solutions to them that I used in the consumer's code.


<h2>Client does not try to reconnect if the initial connection is lost</h2>

UPD: It's stated that it should reconnect now on main branch, but it barely works. The session is dropped if reconnect succeeds. Still you need to know when reconnect happened to restore login session, but it's hidden, and you still would need wrappers around API calls, if not because of timeouts, but because of inconvenient reconnection.
Solution: use your own reliable wrappers, and do your own reconnections.

<b>Solution</b>: put timeouts on API calls - on timeout try to connect to the database; drop the old client on success. Code for connection also needs timeouts, because it's used for authentication and assuming identity, which are API calls.

<h2>No timeouts on API calls from Client</h2>

The calls do not time out.

UPD: Calls timeout, but I don't use _these_ timeouts.

<b>Solution</b>: wrap in timeouts. I'm using <code>tokio::timeout</code>. But I don't know how a problem would be solved for a blocking client more or less gracefully, without modifying the library's source code.

<h2>When Client is idle for a long time it becomes unresponsive</h2>

A "long time" is vaguely specified, but experimentally 2 or more hours would likely do. 10 minutes is too little.

<b>Solution</b>: ping the server from time to time, experimentally 5 minutes works impeccably, but there's a room for an even larger delay.

Conclusion
-
After solving these problems the client becomes responsive and self-healing, and database does not crash so easily.