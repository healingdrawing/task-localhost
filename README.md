# Task "localhost"
grit:lab Åland Islands 2023

## Description
Rust web server, HTTP/1.1 protocol compatible, able to run Python3 CGI scripts.  

**The repository file system structure is important.**  

For details/restrictions see [Task and audit questions](https://github.com/01-edu/public/tree/master/subjects/localhost)

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

### Site customization:

Do not create messy and follow examples style to structure sites. Or just do nothing.
Do not use spaces in the names of any folders and files.
Place inside the `static` folder the folder with the unique name of the site, f.e. `site2`, and inside the site folder place the `index.html` file and so on.  
Follow the examples in the `static` folder on your own risk, or do not touch it.  
The `settings` file must be configured properly to use the site.
The expectable structure of the site can be f.e. next:
- static/site2/favicon.ico
- static/site2/index.html
- static/site2/style/style.css
- static/site2/error/500.html
According to task, the next error pages must be implemented: 400, 403, 404, 405, 413, 500. So you must provide the correct settings file configuration to error page folder. And the error folder must include properly created files: `400.html`, `403.html`, `404.html`, `405.html`, `413.html`, `500.html`.  
Otherwise do not expect server will work/initalize properly.

## Authors
- [healingdrawing](https://healingdrawing.github.io)