# zugzug

Utility to manage working directories

## Basic Usage

Add a bucket to create directories in

```bash
$ zz bucket add tmp $(mktemp -d)
```

Create a new work directory in the default bucket

```bash
$ zz mkdir my_dir
/path/to/bucket/YYYYMMDD_my_dir
```

or in a specific bucket

```bash
$ zz mkdir -b other_bucket my_dir2
/path/to/other_bucket/YYYYMMDD_my_dir2
```

List directories

```bash
$ zz ls
tmp YYYYMMDD my_dir /path/to/bucket/YYYYMMDD_my_dir
```

Set default work bucket

```bash
$ zz default <bucket name>
```

Forget about a bucket. This will stop tracking the bucket, but will not touch
files on disk.

```bash
$ zz bucket forget <name>
```

