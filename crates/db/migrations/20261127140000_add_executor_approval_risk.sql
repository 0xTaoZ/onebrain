-- Why an approval request was judged dangerous, if it was.
ALTER TABLE chat_executor_approval_requests ADD COLUMN risk TEXT;
