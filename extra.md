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
Implemented `equivalent of select`. The `flow.rs ... listener.incoming().for_each_concurrent(None, |stream| async {` section, takes the incoming connection  selected from incomings connections queue `incoming()` which is non-blocking iterator, and manages stream i/o, inside the spawned task.  
So **only one select equivalent is used to read the client requests and write response**.  

---  

> Why is it important to use only one select and how was it achieved?

The `select` is used to wait for multiple futures to complete and then take action based on which future completes first. It can be used to handle multiple I/O operations concurrently, but it requires careful management of the futures and their states.  

The `for_each_concurrent` method is used to handle multiple connections concurrently, which `is similar to what select does`. However, for_each_concurrent is higher-level and easier to use, as it automatically manages the tasks and their states.  

---

> Read the code that goes from the select (or equivalent) to the read and write of a client, is there only one read or write per client per select (or equivalent)?  

The read and write of a client is inside the spawned task, which is spawned for each connection. So there is only one read or write per client per select (or equivalent).  

The `read` implemented in `flow.rs ... read_with_timeout` function.  

The `write` implemented in `flow.rs ... match write_response_into_stream` function.

---

> Are the return values for I/O functions checked properly?  

The return values for I/O functions checked properly.  
The `unwrap`s are managed properly using `match` as replacement.  

---

> If an error is returned by the previous functions on a socket, is the client removed?

The `match` is used to handle errors, and the client is removed.  
The `flow.rs ... task::spawn` section. The end of this section implements the client removal, naturally, at the end of the spawned task. Additionally the `return` used to exit the spawned task in case of errors of the `write`.  

---

> Is writing and reading ALWAYS done through a select (or equivalent)?

The writing and reading is always done through a `equivalent of select`, inside the spawned task `for_each_concurrent` section.

---

> Setup a single server with a single port.  

The `settings` file, configuration with `server_name = "localhost"`.

---

> Setup multiple servers with different port.  

The `settings` file, configuration with `server_name = "default"`. Port `8082`.  

The `settings` file, configuration with `server_name = "mega.company"`. Port `8082`.  

---

> Setup multiple servers with different hostnames (for example: `curl --resolve test.com:80:127.0.0.1 http://test.com/`).  

The `settings` file, configuration with `server_name = "mega.company"`.  

Testing command: `curl --resolve mega.company:8082:127.0.0.2 http://mega.company:8082/uploads`  

Shows the uploads page html content, because method `GET` is `ALLOWED` for uploads in settings.  

The `settings` file, configuration with `server_name = "micro.company"`.  

Testing command: `curl --resolve micro.company:8082:127.0.0.2 http://micro.company:8082/uploads`  

Shows the 405 status code `405.html` page content, because method `GET` is `NOT ALLOWED` for uploads in settings.  

---

> Setup custom error pages.  

The `settings` file. Any configuration `error_pages_prefix` mandatory parameter.

---

> Limit the client body (for example: curl -X POST -H "Content-Type: plain/text" --data "BODY with something shorter or longer than body limit").

The `settings` file, configuration with `server_name = "localhost"` and `client_body_size = 11`.  

Testing command: `curl -X POST -H "Content-Type: application/x-www-form-urlencoded" --data-raw $'hello world' http://localhost:8080/cgi/useless.py/useless_file`.  

Shows
```
Hello from Rust and Python3: PATH_INFO: /home/user/git/task-localhost
The "/home/user/git/task-localhost/cgi/useless_file" is File
```

Testing command: `curl -X POST -H "Content-Type: application/x-www-form-urlencoded" --data-raw $'hello big world' http://localhost:8080/cgi/useless.py/useless_file`.  

Shows the 413 status code `413.html` page content, because the body size is bigger than `client_body_size` in settings.  

---

> Setup routes and ensure they are taken into account.  

According to the task, the routes can be configured using the one or multiple settings:
> Setup routes with one or multiple of the following settings: ...

The next settings are used to configure the routes:
- Define a list of accepted HTTP methods for the route.  
The `settings` file `routes` parameter.  
- Define a default file for the route if the URL is a directory.  
The `settings` file `default_file` parameter.  

---

> Setup a default file in case the path is a directory.

The `settings` file configuration with `server_name = "default"` and `default_file = "default.html"` parameters.  

Testing command: `curl http://127.0.0.2:8086/redirect.html`.  

Shows the `redirect.html` page content.  

Testing command: `curl http://127.0.0.2:8086/redirect.html/`.  

Shows the default page `index.html` content, because path ends with `/` trailing slash, which was decided to be interpreted as a directory. That is common practice, but not a strict rule, and depends on the server implementation.  

---

> Setup a list of accepted methods for a route (for example: try to DELETE something with and without permission).  

The `settings` file configuration with `server_name = "default"` and `routes = { "redirect.html" = [] }` parameters.

Testing command: `curl http://127.0.0.1:8082/redirect.html`.  

Shows status code 405 `405.html` page content, because the `GET` method is not allowed for the `redirect.html` in settings.  

The `settings` file configuration with `server_name = "mega.company"` and `routes = { "redirect.html" = ["GET", "POST", "DELETE"] }` parameters.  

Testing command: `curl http://127.0.0.2:8086/redirect.html`.  

Shows the `redirect.html` page content, because the `GET` method is allowed for the `redirect.html` in settings.  

---


