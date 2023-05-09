<h1>List of imperfections</h1>

I've found some issues that I consider worthy to address in it's source. I provide a list of issues and solutions to them that I used in the consumer's code.

<h2>No timeouts on API calls from Client</h2>

The calls do not time out.

<b>Solution</b>: wrap in timeouts. I'm using <code>tokio::timeout</code>. But I don't know how a problem would be solved for a blocking client more or less gracefully, without modifying the library's source code.

<h2>Client does not try to reconnect if the initial connection is lost</h2>

<b>Solution</b>: put timeouts on API calls - on timeout try to connect to the database; drop the old client on success. Code for connection also needs timeouts, because it's used for authentication and assuming identity, which are API calls.

<h2>When Client is idle for a long time it becomes unresponsive</h2>

A "long time" is vaguely specified, but experimentally 2 or more hours would likely do. 10 minutes is too little.

<b>Solution</b>: ping the server from time to time, experimentally 5 minutes works impeccably, but there's a room for an even larger delay.

 <h2>Server started via CustomServer::listen_on offered with invalid Certificate returns the error:</h2>

```
Error: error from core a transport error occurred: 'Error completing connection with peer: aborted by peer: the application or application protocol caused the connection to be closed during the handshake'

Caused by:
    a transport error occurred: 'Error completing connection with peer: aborted by peer: the application or application protocol caused the connection to be closed during the handshake'
```

The documentation for <code>listen_to</code>: 

```
/// Listens for incoming client connections. Does not return until the
/// server shuts down.
pub async fn listen_on(&self, port: u16) -> Result<(), Error> {}
```
Seems strange that a possibly malicious client can crash a server so easily. \
\
<b>Solution</b>: do not <code>server.listen(port).await<b>?</b></code> as shown in the examples, but handle or ignore the <code>Error::Transport</code> and <code>loop { server.listen(port).await  }</code>

Conclusion
-
After solving these problems the client becomes responsive and self-healing, and database does not crash so easily.