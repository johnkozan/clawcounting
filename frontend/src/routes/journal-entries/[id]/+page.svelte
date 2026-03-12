<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { journalEntries, type JournalEntryDetail, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, RotateCcw } from '@lucide/svelte';

	let entry = $state<JournalEntryDetail | null>(null);
	let loading = $state(true);
	let reverseDialogOpen = $state(false);
	let reverseDate = $state(new Date().toISOString().split('T')[0]);
	let reversing = $state(false);

	onMount(async () => {
		try {
			const res = await journalEntries.get($page.params.id!);
			entry = res.data;
		} catch {
			toast.error('Journal entry not found');
		} finally {
			loading = false;
		}
	});

	async function handleReverse() {
		if (!entry) return;
		reversing = true;
		try {
			const res = await journalEntries.reverse(entry.id, { entry_date: reverseDate });
			toast.success('Entry reversed');
			reverseDialogOpen = false;
			window.location.href = `/journal-entries/${res.data.id}`;
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to reverse entry');
		} finally {
			reversing = false;
		}
	}
</script>

<svelte:head>
	<title>Journal Entry - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/journal-entries" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">Journal Entry</h1>
		{#if entry?.is_reversal}
			<Badge variant="outline">Reversal</Badge>
		{/if}
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if entry}
		<div class="grid gap-6 lg:grid-cols-3">
			<Card.Root class="lg:col-span-2">
				<Card.Header class="pb-2">
					<Card.Title class="text-base">Details</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-3">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Date</p>
							<p class="font-medium">{entry.entry_date}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Reference</p>
							<p class="font-medium">{entry.reference ?? '-'}</p>
						</div>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Description</p>
						<p class="font-medium">{entry.description}</p>
					</div>
					{#if entry.reverses_id}
						<div>
							<p class="text-sm text-muted-foreground">Reverses</p>
							<a href="/journal-entries/{entry.reverses_id}" class="text-sm hover:underline">
								{entry.reverses_id}
							</a>
						</div>
					{/if}
					{#if entry.metadata && Object.keys(entry.metadata).length > 0}
						<div>
							<p class="text-sm text-muted-foreground">Metadata</p>
							<pre class="mt-1 rounded bg-muted p-2 text-xs">{JSON.stringify(entry.metadata, null, 2)}</pre>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="pb-2">
					<Card.Title class="text-base">Actions</Card.Title>
				</Card.Header>
				<Card.Content>
					<Button
						variant="outline"
						class="w-full"
						onclick={() => (reverseDialogOpen = true)}
						disabled={entry.is_reversal}
					>
						<RotateCcw class="mr-2 h-4 w-4" />
						Reverse Entry
					</Button>
					{#if entry.is_reversal}
						<p class="mt-2 text-xs text-muted-foreground">Reversals cannot be reversed.</p>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Lines -->
		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">Lines</Card.Title>
			</Card.Header>
			<Card.Content class="p-0">
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Account</Table.Head>
							<Table.Head>Description</Table.Head>
							<Table.Head class="text-right">Debit</Table.Head>
							<Table.Head class="text-right">Credit</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each entry.lines as line}
							<Table.Row>
								<Table.Cell>
									<a href="/accounts/{line.account_id}" class="hover:underline">
										{#if line.account_number}
											<span class="font-mono text-muted-foreground">{line.account_number}</span>
										{/if}
										{line.account_name ?? line.account_id}
									</a>
								</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">
									{line.description ?? '-'}
								</Table.Cell>
								<Table.Cell class="text-right font-mono">
									{line.display_debit && line.display_debit !== '0.00' ? line.display_debit : ''}
								</Table.Cell>
								<Table.Cell class="text-right font-mono">
									{line.display_credit && line.display_credit !== '0.00' ? line.display_credit : ''}
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</Card.Content>
		</Card.Root>
	{/if}
</div>

<Dialog.Root bind:open={reverseDialogOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header>
			<Dialog.Title>Reverse Journal Entry</Dialog.Title>
			<Dialog.Description>
				This will create a new entry with debits and credits swapped.
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4">
			<div class="space-y-2">
				<Label for="reverseDate">Reversal Date</Label>
				<Input id="reverseDate" type="date" bind:value={reverseDate} required />
			</div>
			<div class="flex justify-end gap-2">
				<Button variant="outline" onclick={() => (reverseDialogOpen = false)}>Cancel</Button>
				<Button onclick={handleReverse} disabled={reversing}>
					{reversing ? 'Reversing...' : 'Reverse'}
				</Button>
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>
