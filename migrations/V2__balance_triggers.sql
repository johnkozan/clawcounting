-- Auto-update account_balances when journal entry lines are inserted.
-- Requires i128_add() custom function to be registered on the connection.
CREATE TRIGGER update_balance_on_insert
AFTER INSERT ON journal_entry_lines
BEGIN
    INSERT INTO account_balances (account_id, period_id, total_debits, total_credits, last_updated)
    VALUES (
        NEW.account_id,
        (SELECT period_id FROM journal_entries WHERE id = NEW.journal_entry_id),
        NEW.debit_amount,
        NEW.credit_amount,
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    )
    ON CONFLICT(account_id, period_id) DO UPDATE SET
        total_debits = i128_add(total_debits, NEW.debit_amount),
        total_credits = i128_add(total_credits, NEW.credit_amount),
        last_updated = strftime('%Y-%m-%dT%H:%M:%fZ', 'now');
END;
