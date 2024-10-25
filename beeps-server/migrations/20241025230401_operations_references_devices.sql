ALTER TABLE operations
    DROP COLUMN node,
    ADD COLUMN device_id BIGINT,
    ADD CONSTRAINT fk_device
        FOREIGN KEY (device_id)
        REFERENCES devices(id);

DROP INDEX IF EXISTS idx_document_node_timestamp_counter_desc;

CREATE INDEX idx_document_id_device_id_timestamp_desc_counter_desc
    ON operations (document_id, device_id, timestamp DESC, counter DESC);
