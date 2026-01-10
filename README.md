BootMgr
===
Change default boot entry

## Windows
-----------------
> If you have more than one grub entry,
> it will only detect the first one.

- List all boot entries(BCDEdit)
- List all boot entries(Grub2)
- Change default boot entry(BCDEdit)
- Change default boot entry(Grub2)

```
Usage: bootmgr.exe <COMMAND>

Commands:
  list
    Options:
      -g, --grub                       List the GRUB boot entries
      -f, --firmware                   List the firmware boot entries
      -d, --description <DESCRIPTION>  Description for the entry of grub
  set
    Options:
      -g, --grub <ENTRY>               Set the GRUB entry by id or index
      -f, --firmware <ENTRY>           Set the firmware entry by identifier
      -d, --description <DESCRIPTION>  Description for the entry of grub
```

# Warning
-----------------
Modifying boot configuration can render your system unbootable. 
Use it on your own risk!
