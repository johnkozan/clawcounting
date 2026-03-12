<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { journalEntries, accounts, type Account, type CreateJournalEntryRequest, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Plus, Trash2 } from '@lucide/svelte';

	interface Line {
		account_id: string;
		debit_amount: string;
		credit_amount: string;
		description: string;
	}

	let accountList = $state<Account[]>([]);
	let loading = $state(false);

	let entryDate = $state(new Date().toISOString().split('T')[0]);
	let description = $state('');
	let reference = $state('');
	let lines = $state<Line[]>([
		{ account_id: '', debit_amount: '', credit_amount: '', description: '' },
		{ account_id: '', debit_amount: '', credit_amount: '', description: '' }
	]);

	const totalDebits = $derived(
		lines.reduce((sum, l) => sum + (parseFloat(l.debit_amount) || 0), 0)
	);
	const totalCredits = $derived(
		lines.reduce((sum, l) => sum + (parseFloat(l.credit_amount) || 0), 0)
	);
	const isBalanced = $derived(
		totalDebits > 0 && Math.abs(totalDebits - totalCredits) < 0.0000001
	);

	onMount(async () => {
		try {
			const res = await accounts.list({ limit: 200, is_active: true });
			accountList = res.data.filter((a) => !a.has_subledger);
		} catch {
			toast.error('Failed to load accounts');
		}
	});

	function addLine() {
		lines = [...lines, { account_id: '', debit_amount: '', credit_amount: '', description: '' }];
	}

	function removeLine(index: number) {
		if (lines.length <= 2) return;
		lines = lines.filter((_, i) => i !== index);
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!isBalanced) {
			toast.error('Entry must be balanced (debits must equal credits)');
			return;
		}

		loading = true;
		try {
			const req: CreateJournalEntryRequest = {
				entry_date: entryDate,
				description,
				reference: reference || undefined,
				lines: lines
					.filter((l) => l.account_id && (l.debit_amount || l.credit_amount))
					.map((l) => ({
						account_id: l.account_id,
						debit_amount: l.debit_amount || undefined,
						credit_amount: l.credit_amount || undefined,
						description: l.description || undefined
					}))
			};
			const res = await journalEntries.create(req);
			toast.success('Journal entry created');
			goto(`/journal-entries/${res.data.id}`);
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to create entry');
		} finally {
			loading = false;
		}
	}

	function getAccountLabel(id: string): string {
		const acc = accountList.find((a) => a.id === id);
		return acc ? `${acc.account_number} - ${acc.name}` : 'Select account';
	}
</script>

<svelte:head>
	<title>New Journal Entry - ClawCounting</title>
</svelte:head>

<div class="mx-auto max-w-4xl space-y-6">
	<div class="flex items-center gap-4">
		<a href="/journal-entries" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">New Journal Entry</h1>
	</div>

	<form onsubmit={handleSubmit}>
		<Card.Root>
			<Card.Content class="space-y-6 pt-6">
				<div class="grid grid-cols-3 gap-4">
					<div class="space-y-2">
						<Label for="date">Date</Label>
						<Input id="date" type="date" bind:value={entryDate} required />
					</div>
					<div class="space-y-2">
						<Label for="desc">Description</Label>
						<Input id="desc" bind:value={description} placeholder="Sale of goods" required />
					</div>
					<div class="space-y-2">
						<Label for="ref">Reference (optional)</Label>
						<Input id="ref" bind:value={reference} placeholder="INV-001" />
					</div>
				</div>

				<!-- Lines -->
				<div class="space-y-3">
					<div class="flex items-center justify-between">
						<Label>Lines</Label>
						<Button type="button" variant="outline" size="sm" onclick={addLine}>
							<Plus class="mr-1 h-3 w-3" />
							Add Line
						</Button>
					</div>

					<div class="rounded-lg border">
						<div class="grid grid-cols-[1fr_120px_120px_1fr_40px] gap-2 border-b bg-muted/50 px-3 py-2 text-sm font-medium text-muted-foreground">
							<div>Account</div>
							<div class="text-right">Debit</div>
							<div class="text-right">Credit</div>
							<div>Description</div>
							<div></div>
						</div>

						{#each lines as line, i}
							<div class="grid grid-cols-[1fr_120px_120px_1fr_40px] gap-2 border-b px-3 py-2 last:border-b-0">
								<Select.Root type="single" value={line.account_id} onValueChange={(v) => { if (v) lines[i].account_id = v; }}>
									<Select.Trigger class="w-full text-sm">
										{getAccountLabel(line.account_id)}
									</Select.Trigger>
									<Select.Content>
										{#each accountList as acc}
											<Select.Item value={acc.id}>
												{acc.account_number} - {acc.name}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
								<Input
									type="number"
									step="any"
									min="0"
									placeholder="0.00"
									bind:value={line.debit_amount}
									class="text-right"
									oninput={() => { if (line.debit_amount) line.credit_amount = ''; }}
								/>
								<Input
									type="number"
									step="any"
									min="0"
									placeholder="0.00"
									bind:value={line.credit_amount}
									class="text-right"
									oninput={() => { if (line.credit_amount) line.debit_amount = ''; }}
								/>
								<Input
									placeholder="Line description"
									bind:value={line.description}
									class="text-sm"
								/>
								<Button
									type="button"
									variant="ghost"
									size="sm"
									onclick={() => removeLine(i)}
									disabled={lines.length <= 2}
									class="px-2"
								>
									<Trash2 class="h-4 w-4 text-muted-foreground" />
								</Button>
							</div>
						{/each}

						<!-- Totals row -->
						<div class="grid grid-cols-[1fr_120px_120px_1fr_40px] gap-2 border-t bg-muted/50 px-3 py-2">
							<div class="text-sm font-medium">Totals</div>
							<div class="text-right font-mono text-sm font-semibold">
								{totalDebits.toFixed(2)}
							</div>
							<div class="text-right font-mono text-sm font-semibold">
								{totalCredits.toFixed(2)}
							</div>
							<div class="text-sm">
								{#if totalDebits > 0 || totalCredits > 0}
									{#if isBalanced}
										<span class="text-emerald-600">Balanced</span>
									{:else}
										<span class="text-destructive">
											Difference: {Math.abs(totalDebits - totalCredits).toFixed(2)}
										</span>
									{/if}
								{/if}
							</div>
							<div></div>
						</div>
					</div>
				</div>

				<div class="flex justify-end gap-2 pt-4">
					<a href="/journal-entries">
						<Button variant="outline" type="button">Cancel</Button>
					</a>
					<Button type="submit" disabled={loading || !isBalanced}>
						{loading ? 'Creating...' : 'Create Entry'}
					</Button>
				</div>
			</Card.Content>
		</Card.Root>
	</form>
</div>
