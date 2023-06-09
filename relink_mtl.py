import os
import glob
import argparse

def process_obj_files(path_to_obj_dir, desired_mtl_filename, cleanup):
    obj_files = glob.glob(os.path.join(path_to_obj_dir, "*.obj"))

    replaced_mtl_files = []

    for obj_file in obj_files:
        with open(obj_file, "r") as f:
            lines = f.readlines()

        with open(obj_file, "w") as f:
            for line in lines:
                if line.startswith("mtllib"):
                    old_mtl_file = line.split()[1]
                    replaced_mtl_files.append(old_mtl_file)
                    f.write(f"mtllib {desired_mtl_filename}\n")
                else:
                    f.write(line)

    if cleanup:
        for mtl_file in set(replaced_mtl_files):
            try:
                os.remove(os.path.join(path_to_obj_dir, mtl_file))
            except FileNotFoundError:
                pass


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--cleanup", action="store_true",
                        help="If specified, delete replaced .mtl files")
    args = parser.parse_args()

    path_to_main_dir = "assets/korok_1"

    desired_mtl_filename = "../../korok_texture_lib.mtl"

    for root, dirs, files in os.walk(path_to_main_dir):
        process_obj_files(root, desired_mtl_filename, args.cleanup)