import os
import hashlib

def find_duplicate_images(folder_path):
  hashes = {}
  duplicates = []

  for root, _, files in os.walk(folder_path):
    for filename in files:
      if filename.endswith('.png'):
        file_path = os.path.join(root, filename)
        with open(file_path, 'rb') as f:
          file_hash = hashlib.md5(f.read()).hexdigest()

        if file_hash in hashes:
          original = hashes[file_hash]
          duplicates.append((original, file_path))
        else:
          hashes[file_hash] = file_path

  return duplicates

# Example usage:
folder_path = 'assets/Animes/'
duplicates = find_duplicate_images(folder_path)
for original, duplicate in duplicates:
  # print(f'# {original} -> {duplicate}')
  print(f'rm "{duplicate}"')

# import os
# import hashlib
# import re

# def find_duplicate_images(folder_path):
#   hashes = {}
#   duplicates = {}

#   for root, _, files in os.walk(folder_path):
#     for filename in files:
#       if filename.endswith('.png'):
#         file_path = os.path.join(root, filename)
#         with open(file_path, 'rb') as f:
#           file_hash = hashlib.md5(f.read()).hexdigest()

#         if file_hash in hashes:
#           duplicates[file_path] = hashes[file_hash]
#         else:
#           hashes[file_hash] = file_path

#   return duplicates  # dict: duplicate_path -> original_path

# def replace_duplicates_in_ts(ts_path, duplicates):
#   with open(ts_path, 'r', encoding='utf-8') as f:
#     content = f.read()

#   # Prepare replacement map: filename only
#   replace_map = {}
#   for dup, orig in duplicates.items():
#     dup_name = os.path.basename(dup)
#     orig_name = os.path.basename(orig)
#     if dup_name != orig_name:
#       replace_map[dup_name] = orig_name

#   # Replace duplicate file names with original file names in the TS content
#   # Using regex to match whole words to avoid partial replacements
#   def replacer(match):
#     name = match.group(0)
#     return replace_map.get(name, name)

#   if replace_map:
#     pattern = re.compile(r'\b(' + '|'.join(re.escape(k) for k in replace_map.keys()) + r')\b')
#     content = pattern.sub(replacer, content)

#     with open(ts_path, 'w', encoding='utf-8') as f:
#       f.write(content)

# # Usage:
# folder_path = 'assets/Animes/'
# ts_path = '../ts/graphics/assetConstants.ts'

# duplicates = find_duplicate_images(folder_path)
# replace_duplicates_in_ts(ts_path, duplicates)
