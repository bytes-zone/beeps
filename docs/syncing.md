# Syncing

All data is represented as an ordered series of operations, kept in order with a hybrid logical clock. This allows writes from multiple devices to come together in a consistent way.

Specifically:

- Pings are an add-only set.
- Tags on pings are a last-write wins register.

Each hybrid logical clock keeps track of a timestamp, a counter, and a node ID. The timestamp is used for ordering, the counter is used to break ties, and the node ID is used to disambiguate between nodes (and as a second tiebreaker, if necessary.)

When syncing, each device calculates the highest timestamp it has seen from every other device. It then asks the server for operations newer than that timestamp. The server responds with all operations that have happened since that time for each device, plus all operations for devices not mentioned, and the device applies them according to the ping/tag rules above.

Doing this ensures everything will eventually be consistent. It also allows for offline operation and should be able to handle P2P updates in the future. It does have a drawback, though: if clocks are not monotonic (that is, they can go backwards) then it's possible for a device to miss an update. This is a known issue with hybrid logical clocks, and it's something we'll have to live with for now. In the future, we may calculate a checksum of the data and compare it to the server's checksum to ensure we have everything.
