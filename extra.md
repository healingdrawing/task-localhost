[Back to README.md](README.md)  

Audit materials for `localhost` project copied in December 2023, from the [https://github.com/01-edu/public/tree/master/subjects/localhost/audit](https://github.com/01-edu/public/tree/master/subjects/localhost/audit), marked as blockquote.  
Horizontal lines added to separate the audit material sections.

---  

> Localhost is about creating your own HTTP server and test it with an actual browser.  

> Take the necessary time to understand the project and to test it, looking into the source code will help a lot.  

---  

> How does an HTTP server works?  

Inside one thread, for each managed port, spawned concurency tasks. Each task implements managing of incoming connections using TcpListener, TcpStream, asynchronious I/O. The `flow.rs ... task::spawn(async move {` section.  

---  

> Which function was used for I/O Multiplexing and how does it works?  

I/O multiplexing is a method that allows a program to monitor multiple I/O channels (like network socket/`TcpListener`) at the same time. Implemented using `flow.rs ... listener.incoming().for_each_concurrent(None, |stream| async {` method used to handle each connection concurrently in a separate task. The `None` means that there is no limit to the number of concurrent tasks(for each connection, a new task is spawned).  

---  

> Is the server using only one select **(or equivalent)** to read the client requests and write answers?

In the context of networking, `select` is a function that is used for I/O multiplexing, which allows a program to monitor multiple I/O channels (like network sockets) at the same time.  
Implemented `equivalent of select`. The `flow.rs ... listener.incoming().for_each_concurrent(None, |stream| async {` section, takes the incoming connection  selected from incomings connections queue ```incoming()``` which is non-blocking iterator, and manages stream i/o, inside the spawned task.  

---

