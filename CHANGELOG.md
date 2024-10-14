# Changelog

## 0.2.2

Turns out that the implementation that fills pings was only looking at pings in the past, so it would always add a new ping when called. This messes up the distribution of pings. I've also added a bunch of tracing to help debug this kind of thing in the future. If you want to see a bunch of internal logic in this release, add `--log-level trace`.

## 0.2.1

Improved the code that calculates the next ping time. It's now way better tested, and the lambda value is set in a more coherent way. The default is now `1 / 45`, which should result in pings every 45 minutes, on average. Previously, this was set to `45 / 60`, which would have resulted in pings every 1.3 hours or so.

## 0.2.0

Convert storage to a list of syncable operations. This is the first step towards syncing between devices, but it does break the data storage from 0.1.0. Please open an issue if you were using 0.1.0 and need a data migration script.

## 0.1.0

Initial release, very rough (on purpose!)
