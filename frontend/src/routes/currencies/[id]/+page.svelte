<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { currencies, type Currency, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft } from '@lucide/svelte';

	let currency = $state<Currency | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let editName = $state('');
	let editSymbol = $state('');

	onMount(async () => {
		try {
			const res = await currencies.get($page.params.id!);
			currency = res.data;
			editName = currency.name;
			editSymbol = currency.symbol;
		} catch {
			toast.error('Currency not found');
		} finally {
			loading = false;
		}
	});

	async function handleSave(e: Event) {
		e.preventDefault();
		if (!currency) return;
		saving = true;
		try {
			const res = await currencies.update(currency.id, { name: editName, symbol: editSymbol });
			currency = res.data;
			toast.success('Currency updated');
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to update');
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>{currency?.code ?? 'Currency'} - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/currencies" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">{currency?.code ?? 'Currency'}</h1>
		{#if currency}
			<Badge variant={currency.asset_type === 'fiat' ? 'default' : 'secondary'}>
				{currency.asset_type}
			</Badge>
		{/if}
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if currency}
		<div class="grid gap-6 lg:grid-cols-2">
			<Card.Root>
				<Card.Header>
					<Card.Title class="text-base">Edit Currency</Card.Title>
				</Card.Header>
				<Card.Content>
					<form onsubmit={handleSave} class="space-y-4">
						<div class="space-y-2">
							<Label for="name">Name</Label>
							<Input id="name" bind:value={editName} required />
						</div>
						<div class="space-y-2">
							<Label for="symbol">Symbol</Label>
							<Input id="symbol" bind:value={editSymbol} required />
						</div>
						<Button type="submit" disabled={saving}>
							{saving ? 'Saving...' : 'Save Changes'}
						</Button>
					</form>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title class="text-base">Details</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-3">
					<div>
						<p class="text-sm text-muted-foreground">Code</p>
						<p class="font-mono">{currency.code}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Asset Scale</p>
						<p>{currency.asset_scale} decimal places</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">CAIP-19 Identifier</p>
						<p class="font-mono text-sm break-all">{currency.caip19_id}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Created</p>
						<p class="text-sm">{currency.created_at}</p>
					</div>
				</Card.Content>
			</Card.Root>
		</div>
	{/if}
</div>
