#!/usr/bin/env python3
import os
import sys
import zipfile
import tempfile
import shutil
from pathlib import Path

def update_wheel_metadata(wheel_path):
    """Update wheel metadata to change package name from terminator-py to terminator"""
    temp_dir = tempfile.mkdtemp()
    
    try:
        # Extract wheel
        with zipfile.ZipFile(wheel_path, 'r') as zip_ref:
            zip_ref.extractall(temp_dir)
        
        # Find and update METADATA file
        for root, dirs, files in os.walk(temp_dir):
            if 'METADATA' in files:
                metadata_path = os.path.join(root, 'METADATA')
                with open(metadata_path, 'r') as f:
                    content = f.read()
                
                # Replace package name
                content = content.replace('Name: terminator-py', 'Name: terminator')
                content = content.replace('terminator-py', 'terminator')
                
                with open(metadata_path, 'w') as f:
                    f.write(content)
            
            # Update dist-info directory name
            if root.endswith('.dist-info') and 'terminator_py' in root:
                new_root = root.replace('terminator_py', 'terminator')
                os.rename(root, new_root)
        
        # Recreate wheel
        new_wheel_path = wheel_path.replace('terminator_py', 'terminator')
        with zipfile.ZipFile(new_wheel_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
            for root, dirs, files in os.walk(temp_dir):
                for file in files:
                    file_path = os.path.join(root, file)
                    arcname = os.path.relpath(file_path, temp_dir)
                    zipf.write(file_path, arcname)
        
        # Remove original wheel if different name
        if new_wheel_path != wheel_path:
            os.remove(wheel_path)
        
        print(f"Updated: {os.path.basename(wheel_path)} -> {os.path.basename(new_wheel_path)}")
        
    finally:
        shutil.rmtree(temp_dir)

def main():
    # Process all wheels in current directory
    for wheel_file in Path('.').glob('*.whl'):
        if 'terminator_py' in str(wheel_file):
            update_wheel_metadata(str(wheel_file))

if __name__ == '__main__':
    main()