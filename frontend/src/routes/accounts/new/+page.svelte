<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { accounts, currencies, type Currency, type CreateAccountRequest, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft } from '@lucide/svelte';

	let currencyList = $state<Currency[]>([]);
	let loading = $state(false);

	let form = $state<CreateAccountRequest>({
		currency_id: '',
		account_number: '',
		name: '',
		account_type: 'asset',
		normal_balance: 'debit',
		has_subledger: false
	});

	onMount(async () => {
		try {
			const res = await currencies.list({ limit: 200 });
			currencyList = res.data;
		} catch {
			toast.error('Failed to load currencies');
		}
	});

	const normalDefaults: Record<string, string> = {
		asset: 'debit',
		expense: 'debit',
		liability: 'credit',
		equity: 'credit',
		revenue: 'credit'
	};

	function onTypeChange(value: string | undefined) {
		if (!value) return;
		form.account_type = value as CreateAccountRequest['account_type'];
		form.normal_balance = normalDefaults[value] as 'debit' | 'credit';
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		loading = true;
		try {
			const res = await accounts.create(form);
			toast.success('Account created');
			goto(`/accounts/${res.data.id}`);
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to create account');
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>New Account - ClawCounting</title>
</svelte:head>

<div class="mx-auto max-w-2xl space-y-6">
	<div class="flex items-center gap-4">
		<a href="/accounts" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">New Account</h1>
	</div>

	<Card.Root>
		<Card.Content class="pt-6">
			<form onsubmit={handleSubmit} class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="number">Account Number</Label>
						<Input id="number" bind:value={form.account_number} placeholder="1000" required />
					</div>
					<div class="space-y-2">
						<Label for="name">Account Name</Label>
						<Input id="name" bind:value={form.name} placeholder="Cash" required />
					</div>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label>Account Type</Label>
						<Select.Root type="single" value={form.account_type} onValueChange={onTypeChange}>
							<Select.Trigger class="w-full">
								{form.account_type ? form.account_type.charAt(0).toUpperCase() + form.account_type.slice(1) : 'Select type'}
							</Select.Trigger>
							<Select.Content>
								<Select.Item value="asset">Asset</Select.Item>
								<Select.Item value="liability">Liability</Select.Item>
								<Select.Item value="equity">Equity</Select.Item>
								<Select.Item value="revenue">Revenue</Select.Item>
								<Select.Item value="expense">Expense</Select.Item>
							</Select.Content>
						</Select.Root>
					</div>
					<div class="space-y-2">
						<Label>Normal Balance</Label>
						<Select.Root type="single" value={form.normal_balance} onValueChange={(v) => { if (v) form.normal_balance = v as 'debit' | 'credit'; }}>
							<Select.Trigger class="w-full">
								{form.normal_balance ? form.normal_balance.charAt(0).toUpperCase() + form.normal_balance.slice(1) : 'Select'}
							</Select.Trigger>
							<Select.Content>
								<Select.Item value="debit">Debit</Select.Item>
								<Select.Item value="credit">Credit</Select.Item>
							</Select.Content>
						</Select.Root>
					</div>
				</div>

				<div class="space-y-2">
					<Label>Currency</Label>
					<Select.Root type="single" value={form.currency_id} onValueChange={(v) => { if (v) form.currency_id = v; }}>
						<Select.Trigger class="w-full">
							{currencyList.find((c) => c.id === form.currency_id)?.code ?? 'Select currency'}
						</Select.Trigger>
						<Select.Content>
							{#each currencyList as c}
								<Select.Item value={c.id}>{c.code} - {c.name}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<div class="space-y-2">
					<Label for="xbrl">XBRL Tag (optional)</Label>
					<Input id="xbrl" bind:value={form.xbrl_tag} placeholder="us-gaap:CashAndCashEquivalents" />
				</div>

				<div class="flex items-center gap-2">
					<input
						type="checkbox"
						id="subledger"
						bind:checked={form.has_subledger}
						class="rounded border-input"
					/>
					<Label for="subledger">Control account (has subledger)</Label>
				</div>

				<div class="flex justify-end gap-2 pt-4">
					<a href="/accounts">
						<Button variant="outline" type="button">Cancel</Button>
					</a>
					<Button type="submit" disabled={loading}>
						{loading ? 'Creating...' : 'Create Account'}
					</Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>
</div>
