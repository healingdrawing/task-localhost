# stupid.py
import os
import sys

# Get the PATH_INFO variable
path_info = os.environ.get('PATH_INFO', '')

# The file to process is the first argument to the script
file_to_process = sys.argv[1]

# Combine the script's location with the PATH_INFO to get the full path
full_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), path_info, file_to_process)

# Check if the full_path is a file, folder, or does not exist
if os.path.isfile(full_path):
   print("File")
elif os.path.isdir(full_path):
   print("Folder")
else:
   print("Wrong path")
