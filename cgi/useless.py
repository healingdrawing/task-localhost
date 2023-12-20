# stupid.py
import os
import sys

# Get the PATH_INFO variable
path_info = os.environ.get('PATH_INFO', '')
print("PATH_INFO: " + path_info)

# The file to process is the first argument to the script
file_to_process = sys.argv[1]

# Combine the script's location with the PATH_INFO to get the full path
full_path = os.path.join(path_info, "cgi", file_to_process)

# Check if the full_path is a file, folder, or does not exist
if os.path.isfile(full_path):
   print("The \""+ full_path +"\" is File")
elif os.path.isdir(full_path):
   print("The \""+ full_path +"\" is Folder")
else:
   print("The \""+ full_path +"\" is Wrong path")
