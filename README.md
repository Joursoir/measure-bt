# BIOS Boot Time Measurement

This UEFI utility will help you measure the BIOS boot time. Works with **all BIOS** types, including AMI and custom firmware.

### Supported States

* **Cold Reset**: Supported
* **Warm Reset**: Not implemented

## Setup Instructions

### Step 1: Create a Bootable USB Drive

1. Insert a USB drive into your computer.

2. Format the drive with a FAT32 partition:

```
# fdisk /dev/sdX
```

Replace `/dev/sdX` with your USB device. Follow these steps:

* Press `g` to create a new empty GPT partition table.
* Press `n` to create a new partition.
* Select default values for partition number, start, and end sectors.
* Press `t` and type `uefi` to change partition type to `EFI System`.
* Press `w` to write changes.

Format the partition:

```
# mkfs.fat -F 32 /dev/sdX1
```

Replace `/dev/sdX1` with the partition name.

3. Mount the USB drive:

```
# mount /dev/sdX1 /mnt
```

4. Create the following directory structure on the USB drive:

```
$ mkdir -p /mnt/efi/boot
```

5. Place the UEFI utility in the /efi/boot/ directory and rename it:

```
$ mv measure-bt.efi /mnt/efi/boot/bootx64.efi
```

6. Unmount the USB drive:

```
# sudo umount /mnt
```

### Step 2: Configure Boot Sequence

1. Insert the USB drive into the test platform.

2. Enter the BIOS Setup Utility and set the USB drive as the default boot device.

3. Save and restart BIOS.

### Step 3: Analyze Boot Times

After completing X amount of test cycles, the utility will write the measurement results to the root directory of the USB drive in a file named `measure_count.txt`. And pass boot control to the next available boot option.

Copy the file and run the Analysis Script:

```
$ python calculate_boot_time.py measure_count.txt
```

The script outputs the average boot time based on the data. Example output:

```
[21, 19, 19, 19, 19, 19, 19, ...]
The average difference between boot times is 19.01 seconds, based on 125 reboots.
```

## Compilation Instructions

```
$ cargo build --target x86_64-unknown-uefi
```
