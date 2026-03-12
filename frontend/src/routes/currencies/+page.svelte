<script lang="ts">
	import { onMount } from 'svelte';
	import { currencies, type Currency, type CreateCurrencyRequest, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { Plus, Search } from '@lucide/svelte';
	import fiatCurrencies from '$lib/data/fiat-currencies.json';

	let items = $state<Currency[]>([]);
	let loading = $state(true);
	let hasMore = $state(false);
	let nextCursor = $state<string | undefined>();

	// Add fiat dialog
	let fiatDialogOpen = $state(false);
	let fiatSearch = $state('');
	let fiatLoading = $state<string | null>(null);

	// Add custom dialog
	let customDialogOpen = $state(false);
	let customForm = $state<CreateCurrencyRequest>({
		code: '',
		name: '',
		symbol: '',
		asset_scale: 2,
		asset_type: 'crypto',
		caip19_id: ''
	});
	let customLoading = $state(false);

	const filteredFiat = $derived(
		fiatCurrencies.filter(
			(c) =>
				c.code.toLowerCase().includes(fiatSearch.toLowerCase()) ||
				c.name.toLowerCase().includes(fiatSearch.toLowerCase())
		)
	);

	onMount(loadCurrencies);

	async function loadCurrencies() {
		loading = true;
		try {
			const res = await currencies.list({ limit: 200 });
			items = res.data;
			hasMore = res.has_more;
			nextCursor = res.next_cursor;
		} catch {
			toast.error('Failed to load currencies');
		} finally {
			loading = false;
		}
	}

	async function addFiat(fiat: (typeof fiatCurrencies)[number]) {
		fiatLoading = fiat.code;
		try {
			await currencies.create({
				code: fiat.code,
				name: fiat.name,
				symbol: fiat.symbol,
				asset_scale: fiat.asset_scale,
				asset_type: 'fiat',
				caip19_id: fiat.caip19_id
			});
			toast.success(`${fiat.code} added`);
			await loadCurrencies();
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to add currency');
		} finally {
			fiatLoading = null;
		}
	}

	async function addCustom(e: Event) {
		e.preventDefault();
		customLoading = true;
		try {
			await currencies.create(customForm);
			toast.success(`${customForm.code} added`);
			customDialogOpen = false;
			customForm = { code: '', name: '', symbol: '', asset_scale: 2, asset_type: 'crypto', caip19_id: '' };
			await loadCurrencies();
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to add currency');
		} finally {
			customLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Currencies - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Currencies</h1>
		<div class="flex gap-2">
			<Button variant="outline" onclick={() => (fiatDialogOpen = true)}>
				<Plus class="mr-2 h-4 w-4" />
				Add Fiat
			</Button>
			<Button onclick={() => (customDialogOpen = true)}>
				<Plus class="mr-2 h-4 w-4" />
				Add Custom
			</Button>
		</div>
	</div>

	<Card.Root>
		<Card.Content class="p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Code</Table.Head>
						<Table.Head>Name</Table.Head>
						<Table.Head>Symbol</Table.Head>
						<Table.Head>Scale</Table.Head>
						<Table.Head>Type</Table.Head>
						<Table.Head>CAIP-19</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#if loading}
						<Table.Row>
							<Table.Cell colspan={6} class="text-center text-muted-foreground">
								Loading...
							</Table.Cell>
						</Table.Row>
					{:else if items.length === 0}
						<Table.Row>
							<Table.Cell colspan={6} class="text-center text-muted-foreground">
								No currencies configured. Add a fiat or custom currency to get started.
							</Table.Cell>
						</Table.Row>
					{:else}
						{#each items as currency}
							<Table.Row class="cursor-pointer hover:bg-muted/50">
								<Table.Cell>
									<a href="/currencies/{currency.id}" class="font-medium hover:underline">
										{currency.code}
									</a>
								</Table.Cell>
								<Table.Cell>{currency.name}</Table.Cell>
								<Table.Cell>{currency.symbol}</Table.Cell>
								<Table.Cell>{currency.asset_scale}</Table.Cell>
								<Table.Cell>
									<Badge variant={currency.asset_type === 'fiat' ? 'default' : 'secondary'}>
										{currency.asset_type}
									</Badge>
								</Table.Cell>
								<Table.Cell class="font-mono text-xs text-muted-foreground">
									{currency.caip19_id}
								</Table.Cell>
							</Table.Row>
						{/each}
					{/if}
				</Table.Body>
			</Table.Root>
		</Card.Content>
	</Card.Root>
</div>

<!-- Add Fiat Dialog -->
<Dialog.Root bind:open={fiatDialogOpen}>
	<Dialog.Content class="max-h-[80vh] sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Add Fiat Currency</Dialog.Title>
			<Dialog.Description>Select a currency from the ISO 4217 standard.</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4">
			<div class="relative">
				<Search class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
				<Input placeholder="Search currencies..." bind:value={fiatSearch} class="pl-9" />
			</div>
			<div class="max-h-80 overflow-y-auto space-y-1">
				{#each filteredFiat.slice(0, 50) as fiat}
					<button
						class="flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm hover:bg-muted"
						onclick={() => addFiat(fiat)}
						disabled={fiatLoading === fiat.code}
					>
						<div>
							<span class="font-medium">{fiat.code}</span>
							<span class="text-muted-foreground"> - {fiat.name}</span>
						</div>
						<span class="text-muted-foreground">{fiat.symbol}</span>
					</button>
				{/each}
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>

<!-- Add Custom Dialog -->
<Dialog.Root bind:open={customDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Add Custom Currency</Dialog.Title>
			<Dialog.Description>Add a cryptocurrency or custom token.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={addCustom} class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="code">Code</Label>
					<Input id="code" bind:value={customForm.code} placeholder="ETH" required />
				</div>
				<div class="space-y-2">
					<Label for="symbol">Symbol</Label>
					<Input id="symbol" bind:value={customForm.symbol} placeholder="ETH" required />
				</div>
			</div>
			<div class="space-y-2">
				<Label for="name">Name</Label>
				<Input id="name" bind:value={customForm.name} placeholder="Ether" required />
			</div>
			<div class="space-y-2">
				<Label for="scale">Asset Scale (decimal places)</Label>
				<Input id="scale" type="number" min="0" max="18" bind:value={customForm.asset_scale} required />
			</div>
			<div class="space-y-2">
				<Label for="caip19">CAIP-19 Identifier</Label>
				<Input id="caip19" bind:value={customForm.caip19_id} placeholder="eip155:1/slip44:60" required />
			</div>
			<div class="flex justify-end gap-2">
				<Button variant="outline" type="button" onclick={() => (customDialogOpen = false)}>
					Cancel
				</Button>
				<Button type="submit" disabled={customLoading}>
					{customLoading ? 'Adding...' : 'Add Currency'}
				</Button>
			</div>
		</form>
	</Dialog.Content>
</Dialog.Root>
