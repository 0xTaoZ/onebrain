-- Snapshot taken when the run ends, so undo can restore only the files the run changed.
ALTER TABLE chat_run_checkpoints ADD COLUMN after_checkpoint TEXT;
