import argparse

parser = argparse.ArgumentParser(description="Calculate the average difference between boot times.")
parser.add_argument('file_path', type=str, help="Path to the file containing boot times, should be generate by measure-bt.efi")

args = parser.parse_args()

# Read the data from the file and parse the boot times
try:
    with open(args.file_path, 'r') as file:
        lines = file.readlines()
        times = [int(line.split(": ")[1].split()[0]) for line in lines]
except FileNotFoundError:
    print(f"Error: File '{args.file_path}' not found.")
    exit(1)
except ValueError:
    print("Error: File contents are not in the expected format.")
    exit(1)

# Calculate the absolute differences between consecutive boots
differences = [abs(times[i] - times[i+1]) for i in range(len(times) - 1)]
print(differences)

average_difference = sum(differences) / len(differences)
print(f"The average difference between boot times is {average_difference:.2f} seconds, based on {len(times)} reboots.")
