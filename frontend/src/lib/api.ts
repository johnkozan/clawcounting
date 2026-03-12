const API_BASE = '/api/v1';
const AUTH_BASE = '/auth';

export class ApiError extends Error {
	code: string;
	status: number;
	field?: string;
	suggestion?: string;

	constructor(status: number, body: { code: string; message: string; field?: string; suggestion?: string }) {
		super(body.message);
		this.code = body.code;
		this.status = status;
		this.field = body.field;
		this.suggestion = body.suggestion;
	}
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
	const token = localStorage.getItem('token');
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};
	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const res = await fetch(path, {
		method,
		headers,
		body: body ? JSON.stringify(body) : undefined
	});

	if (!res.ok) {
		const errorBody = await res.json().catch(() => ({
			code: 'UNKNOWN',
			message: `HTTP ${res.status}`
		}));
		throw new ApiError(res.status, errorBody);
	}

	if (res.status === 204) return undefined as T;
	return res.json();
}

// Auth endpoints (no /api/v1 prefix)
// Backend wraps all responses in { "data": ... }
export const auth = {
	login: async (email: string, password: string) => {
		const res = await request<{ data: { access_token: string; refresh_token: string; token_type: string; expires_in: number } }>('POST', `${AUTH_BASE}/login`, { email, password });
		return res.data;
	},
	refresh: async (refresh_token: string) => {
		const res = await request<{ data: { access_token: string; token_type: string; expires_in: number } }>('POST', `${AUTH_BASE}/refresh`, { refresh_token });
		return res.data;
	},
	me: () =>
		request<{ data: User }>('GET', `${AUTH_BASE}/me`)
};

// API endpoints
export const currencies = {
	list: (params?: { limit?: number; cursor?: string }) =>
		request<ListResponse<Currency>>('GET', `${API_BASE}/currencies${toQuery(params)}`),
	get: (id: string) =>
		request<{ data: Currency }>('GET', `${API_BASE}/currencies/${id}`),
	create: (data: CreateCurrencyRequest) =>
		request<{ data: Currency }>('POST', `${API_BASE}/currencies`, data),
	update: (id: string, data: UpdateCurrencyRequest) =>
		request<{ data: Currency }>('PATCH', `${API_BASE}/currencies/${id}`, data)
};

export const accounts = {
	list: (params?: { limit?: number; cursor?: string; account_type?: string; currency_id?: string; is_active?: boolean; parent_id?: string }) =>
		request<ListResponse<Account>>('GET', `${API_BASE}/accounts${toQuery(params)}`),
	get: (id: string) =>
		request<{ data: Account }>('GET', `${API_BASE}/accounts/${id}`),
	create: (data: CreateAccountRequest) =>
		request<{ data: Account }>('POST', `${API_BASE}/accounts`, data),
	update: (id: string, data: UpdateAccountRequest) =>
		request<{ data: Account }>('PATCH', `${API_BASE}/accounts/${id}`, data),
	subAccounts: (id: string) =>
		request<ListResponse<Account>>('GET', `${API_BASE}/accounts/${id}/sub-accounts`),
	balance: (id: string, periodId?: string) =>
		request<{ data: AccountBalance }>('GET', `${API_BASE}/accounts/${id}/balance${toQuery({ period_id: periodId })}`),
	transactions: (id: string, params?: { limit?: number; cursor?: string }) =>
		request<ListResponse<JournalEntryLine>>('GET', `${API_BASE}/accounts/${id}/transactions${toQuery(params)}`)
};

export const journalEntries = {
	list: (params?: { limit?: number; cursor?: string; period_id?: string; start_date?: string; end_date?: string; account_id?: string }) =>
		request<ListResponse<JournalEntry>>('GET', `${API_BASE}/journal-entries${toQuery(params)}`),
	get: (id: string) =>
		request<{ data: JournalEntryDetail }>('GET', `${API_BASE}/journal-entries/${id}`),
	create: (data: CreateJournalEntryRequest) =>
		request<{ data: JournalEntryDetail }>('POST', `${API_BASE}/journal-entries`, data),
	reverse: (id: string, data?: { entry_date?: string }) =>
		request<{ data: JournalEntryDetail }>('POST', `${API_BASE}/journal-entries/${id}/reverse`, data)
};

export const periods = {
	list: (params?: { limit?: number; cursor?: string }) =>
		request<ListResponse<FinancialPeriod>>('GET', `${API_BASE}/periods${toQuery(params)}`),
	get: (id: string) =>
		request<{ data: FinancialPeriod }>('GET', `${API_BASE}/periods/${id}`),
	create: (data: CreatePeriodRequest) =>
		request<{ data: FinancialPeriod }>('POST', `${API_BASE}/periods`, data),
	close: (id: string, preview = false) =>
		request<{ data: ClosingResult }>('POST', `${API_BASE}/periods/${id}/close${preview ? '?preview=true' : ''}`)
};

export const reports = {
	trialBalance: (params?: { period_id?: string; currency_id?: string }) =>
		request<{ data: TrialBalanceReport }>('GET', `${API_BASE}/reports/trial-balance${toQuery(params)}`),
	balanceSheet: (params?: { period_id?: string; as_of_date?: string }) =>
		request<{ data: BalanceSheetReport }>('GET', `${API_BASE}/reports/balance-sheet${toQuery(params)}`),
	incomeStatement: (params: { period_id: string }) =>
		request<{ data: IncomeStatementReport }>('GET', `${API_BASE}/reports/income-statement${toQuery(params)}`),
	generalLedger: (params: { account_id: string; period_id?: string; start_date?: string; end_date?: string; sort?: string; limit?: number; cursor?: string }) =>
		request<{ data: GeneralLedgerReport }>('GET', `${API_BASE}/reports/general-ledger${toQuery(params)}`)
};

export const settings = {
	get: () =>
		request<{ data: Settings }>('GET', `${API_BASE}/settings`),
	update: (data: UpdateSettingsRequest) =>
		request<{ data: Settings }>('PATCH', `${API_BASE}/settings`, data)
};

export const users = {
	list: (params?: { limit?: number; cursor?: string }) =>
		request<ListResponse<User>>('GET', `${API_BASE}/users${toQuery(params)}`),
	get: (id: string) =>
		request<{ data: User }>('GET', `${API_BASE}/users/${id}`),
	create: (data: CreateUserRequest) =>
		request<{ data: User }>('POST', `${API_BASE}/users`, data),
	createServiceAccount: (data: CreateServiceAccountRequest) =>
		request<{ data: User & { api_key: string } }>('POST', `${API_BASE}/users/service-accounts`, data),
	update: (id: string, data: UpdateUserRequest) =>
		request<{ data: User }>('PATCH', `${API_BASE}/users/${id}`, data)
};

function toQuery(params?: Record<string, unknown>): string {
	if (!params) return '';
	const entries = Object.entries(params).filter(([, v]) => v !== undefined && v !== null);
	if (entries.length === 0) return '';
	return '?' + new URLSearchParams(entries.map(([k, v]) => [k, String(v)])).toString();
}

// Types
export interface ListResponse<T> {
	data: T[];
	has_more: boolean;
	next_cursor?: string;
}

export interface Currency {
	id: string;
	code: string;
	name: string;
	symbol: string;
	asset_scale: number;
	asset_type: 'fiat' | 'crypto';
	caip19_id: string;
	created_at: string;
	updated_at: string;
}

export interface CreateCurrencyRequest {
	code: string;
	name: string;
	symbol: string;
	asset_scale: number;
	asset_type: 'fiat' | 'crypto';
	caip19_id: string;
}

export interface UpdateCurrencyRequest {
	name?: string;
	symbol?: string;
}

export interface Account {
	id: string;
	currency_id: string;
	account_number: string;
	name: string;
	account_type: 'asset' | 'liability' | 'equity' | 'revenue' | 'expense';
	normal_balance: 'debit' | 'credit';
	has_subledger: boolean;
	parent_id?: string;
	entity_id?: string;
	xbrl_tag?: string;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

export interface CreateAccountRequest {
	currency_id: string;
	account_number: string;
	name: string;
	account_type: 'asset' | 'liability' | 'equity' | 'revenue' | 'expense';
	normal_balance: 'debit' | 'credit';
	has_subledger?: boolean;
	parent_id?: string;
	entity_id?: string;
	xbrl_tag?: string;
}

export interface UpdateAccountRequest {
	name?: string;
	is_active?: boolean;
	xbrl_tag?: string;
}

export interface AccountBalance {
	account_id: string;
	period_id?: string;
	total_debits: string;
	total_credits: string;
	net_balance: string;
	display_debits: string;
	display_credits: string;
	display_balance: string;
}

export interface JournalEntry {
	id: string;
	period_id: string;
	entry_date: string;
	description: string;
	reference?: string;
	is_reversal: boolean;
	reverses_id?: string;
	created_by: string;
	created_at: string;
	metadata?: Record<string, unknown>;
}

export interface JournalEntryLine {
	id: string;
	journal_entry_id: string;
	account_id: string;
	account_name?: string;
	account_number?: string;
	debit_amount: string;
	credit_amount: string;
	display_debit?: string;
	display_credit?: string;
	description?: string;
}

export interface JournalEntryDetail extends JournalEntry {
	lines: JournalEntryLine[];
}

export interface CreateJournalEntryRequest {
	entry_date: string;
	description: string;
	reference?: string;
	metadata?: Record<string, unknown>;
	lines: {
		account_id: string;
		debit_amount?: string;
		credit_amount?: string;
		description?: string;
	}[];
}

export interface FinancialPeriod {
	id: string;
	name: string;
	start_date: string;
	end_date: string;
	closed_at?: string;
	closed_by?: string;
	closing_entry_id?: string;
	created_at: string;
}

export interface CreatePeriodRequest {
	name: string;
	start_date: string;
	end_date: string;
}

export interface ClosingResult {
	period: FinancialPeriod;
	closing_entry?: JournalEntryDetail;
}

export interface TrialBalanceReport {
	period_id?: string;
	currency_id?: string;
	rows: {
		account_id: string;
		account_number: string;
		account_name: string;
		account_type: string;
		debit_total: string;
		credit_total: string;
		display_debit: string;
		display_credit: string;
	}[];
	total_debits: string;
	total_credits: string;
	display_total_debits: string;
	display_total_credits: string;
}

export interface BalanceSheetReport {
	as_of_date?: string;
	period_id?: string;
	sections: {
		name: string;
		account_type: string;
		rows: {
			account_id: string;
			account_number: string;
			account_name: string;
			balance: string;
			display_balance: string;
		}[];
		total: string;
		display_total: string;
	}[];
	total_assets: string;
	total_liabilities: string;
	total_equity: string;
	display_total_assets: string;
	display_total_liabilities: string;
	display_total_equity: string;
}

export interface IncomeStatementReport {
	period_id: string;
	sections: {
		name: string;
		account_type: string;
		rows: {
			account_id: string;
			account_number: string;
			account_name: string;
			amount: string;
			display_amount: string;
		}[];
		total: string;
		display_total: string;
	}[];
	net_income: string;
	display_net_income: string;
}

export interface GeneralLedgerReport {
	account_id: string;
	account_name: string;
	starting_balance: string;
	display_starting_balance: string;
	entries: {
		line_id: string;
		entry_id: string;
		entry_date: string;
		description: string;
		reference?: string;
		debit: string;
		credit: string;
		display_debit: string;
		display_credit: string;
		running_balance: string;
		display_running_balance: string;
	}[];
	has_more: boolean;
	next_cursor?: string;
}

export interface Settings {
	instance_name: string;
	retained_earnings_account_id?: string;
	[key: string]: unknown;
}

export interface UpdateSettingsRequest {
	instance_name?: string;
	retained_earnings_account_id?: string;
}

export interface User {
	id: string;
	name: string;
	email?: string;
	user_type: 'human' | 'service';
	permissions: Record<string, boolean>;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

export interface CreateUserRequest {
	name: string;
	email: string;
	password: string;
	permissions?: Record<string, boolean>;
}

export interface CreateServiceAccountRequest {
	name: string;
	permissions?: Record<string, boolean>;
}

export interface UpdateUserRequest {
	name?: string;
	is_active?: boolean;
	permissions?: Record<string, boolean>;
}
