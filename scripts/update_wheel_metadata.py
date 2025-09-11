#!/usr/bin/env python3
import os
import sys
import zipfile
import tempfile
import shutil
import re
from pathlib import Path

def update_wheel_metadata(wheel_path):
    """Update wheel metadata to change package name from terminator-py to terminator"""
    print(f"Processing wheel: {wheel_path}")
    temp_dir = tempfile.mkdtemp()
    
    try:
        # Extract wheel
        with zipfile.ZipFile(wheel_path, 'r') as zip_ref:
            zip_ref.extractall(temp_dir)
        
        # Track if we need to update dist-info directory
        dist_info_renamed = False
        
        # First pass: rename dist-info directories
        for root, dirs, files in os.walk(temp_dir):
            for d in dirs[:]:  # Create a copy of dirs to iterate
                if d.endswith('.dist-info') and 'terminator_py' in d:
                    old_path = os.path.join(root, d)
                    new_name = d.replace('terminator_py', 'terminator')
                    new_path = os.path.join(root, new_name)
                    print(f"  Renaming dist-info: {d} -> {new_name}")
                    os.rename(old_path, new_path)
                    dist_info_renamed = True
                    dirs.remove(d)
                    dirs.append(new_name)
        
        # Second pass: update metadata files
        for root, dirs, files in os.walk(temp_dir):
            # Update METADATA file
            if 'METADATA' in files:
                metadata_path = os.path.join(root, 'METADATA')
                with open(metadata_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Replace package name more carefully
                original_content = content
                content = re.sub(r'^Name: terminator[-_]py$', 'Name: terminator', content, flags=re.MULTILINE)
                content = content.replace('terminator-py', 'terminator')
                
                if content != original_content:
                    print(f"  Updated METADATA in {os.path.relpath(root, temp_dir)}")
                    with open(metadata_path, 'w', encoding='utf-8') as f:
                        f.write(content)
            
            # Update RECORD file
            if 'RECORD' in files:
                record_path = os.path.join(root, 'RECORD')
                with open(record_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                content = content.replace('terminator_py', 'terminator')
                
                with open(record_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"  Updated RECORD in {os.path.relpath(root, temp_dir)}")
            
            # Update top_level.txt if it exists
            if 'top_level.txt' in files:
                top_level_path = os.path.join(root, 'top_level.txt')
                with open(top_level_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Only update if it specifically references terminator_py
                if 'terminator_py' in content:
                    content = content.replace('terminator_py', 'terminator')
                    with open(top_level_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    print(f"  Updated top_level.txt")
        
        # Create new wheel with updated name
        new_wheel_path = wheel_path.replace('terminator_py', 'terminator')
        print(f"  Creating new wheel: {new_wheel_path}")
        
        with zipfile.ZipFile(new_wheel_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
            for root, dirs, files in os.walk(temp_dir):
                for file in files:
                    file_path = os.path.join(root, file)
                    arcname = os.path.relpath(file_path, temp_dir)
                    zipf.write(file_path, arcname)
        
        # Remove original wheel if different name
        if new_wheel_path != wheel_path and os.path.exists(wheel_path):
            os.remove(wheel_path)
            print(f"  Removed original wheel: {wheel_path}")
        
        print(f"✓ Successfully processed: {os.path.basename(new_wheel_path)}")
        
    except Exception as e:
        print(f"✗ Error processing {wheel_path}: {e}")
        raise
    finally:
        shutil.rmtree(temp_dir)

def main():
    # Process all wheels in current directory
    wheels_found = list(Path('.').glob('*.whl'))
    
    if not wheels_found:
        print("No wheel files found in current directory")
        return
    
    print(f"Found {len(wheels_found)} wheel file(s)")
    
    for wheel_file in wheels_found:
        wheel_name = str(wheel_file)
        if 'terminator_py' in wheel_name or 'terminator-py' in wheel_name:
            update_wheel_metadata(wheel_name)
        else:
            print(f"Skipping {wheel_name} - already using 'terminator' name")

if __name__ == '__main__':
    main()