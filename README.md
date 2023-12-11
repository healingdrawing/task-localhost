# Task "localhost"
grit:lab Åland Islands 2023

## Description
Rust web server, HTTP/1.1 protocol compatible, able to run Python3 CGI scripts.  

**The repository file system structure is important.**  

For details/restrictions see [task and audit questions](https://github.com/01-edu/public/tree/master/subjects/localhost)

## Usage
- open terminal in the repository root folder(the `README.md` location)

### Build the project:
- terminal: `./do`

### Run the project:
- terminal: `./runme`

### Development run the project(build and run):
- terminal: `./devrun`

### How server works:

- after build the project, the binary file `runme` will be created in the repository root folder.
- after execute `./runme` the server will try to start, according to the `settrings` file, which follows the TOML format. If any error occurs in initialization, the server will stop and print the error message in the terminal.
- after start the server will listen the ip:port configurations from the `settings` file, which will be printed in the terminal, like `Server` instances.

### Details and restrictions:

- `error 403 Forbidden`,  handling implemeted for case of access with directory uri with not GET method, as a way to enforce that only GET requests are allowed for directory URIs. This status code is commonly used when the server does not wish to reveal exactly why the request has been refused, or when no other response is applicable.  
To test it, you can use the `curl` commands in the terminal.  

Correct case using GET method (return default file as task requires):  
`
curl -X GET http://localhost:8080/
`  

Forbidden case using POST method (return 403 error page):  
`
curl -X POST http://localhost:8080/
`

### Customization:

- to use the executable separately from the project(not recommended), you need to keep in one folder:
- - the executable file `runme`.
- - the `static` folder, includes the sites files.
- - the `cgi` folder, includes the Python3 CGI script.
According to the task requirements only one script implemented and it hardcoded in the `runme` file. So no field for experiments with this old and insecure technology.
- - the `uploads` folder, used to manage file uploads/deletions/shows for servers.  
It is not a static part of the project, and managed separately, to prevent extra activity. This folder includes `.gitignore` file, to prevent uploading files to the repository. Checking for `.gitignore` file is hardcoded into the `runme` file.
For any configuration you can use `/uploads` to uploads page, which is hardcoded into the `runme` file.
- - the `settings` file, configured properly.  
Follow the examples in the `settings` file on your own risk, or do not touch it(it is educational project, aimed to satisfy task requirements in strictly limited time period).  

So finally it should looks like:
- `some_folder/runme` (executable)
- `some_folder/static` (folder)
- `some_folder/cgi` (folder)
- `some_folder/uploads` (folder)
- `some_folder/settings` (file)

### Site customization:

Do not create messy and follow examples style to structure sites. Or just do nothing.
Do not use spaces in the names of any folders and files.  

According to task methods `GET`, `POST`, `DELETE` must be implemented. 
Do not use other methods to configure the server routes.  

Place inside the `static` folder the folder with the unique name of the site, f.e. `site2`, and inside the site folder place the `index.html` file and so on.  
Follow the examples in the `static` folder on your own risk, or do not touch it.  

The `settings` file must be configured properly to use the site.
The expectable structure of the site can be f.e. next:
- static/site2/favicon.ico
- static/site2/index.html
- static/site2/style/style.css
- static/site2/error/500.html  
According to task, the next error pages must be implemented: 400, 403, 404, 405, 413, 500.  
So you must provide the correct settings file configuration to error pages folder. And the error pages folder must contain properly created files: `400.html`, `403.html`, `404.html`, `405.html`, `413.html`, `500.html`.  

Otherwise do not expect server will work/initalize properly.

## Authors
- [healingdrawing](https://healingdrawing.github.io)
