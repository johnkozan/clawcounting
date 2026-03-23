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
	import * as Tabs from '$lib/components/ui/tabs';
	import { toast } from 'svelte-sonner';
	import { Plus, Search, Upload, Check } from '@lucide/svelte';
	import fiatCurrencies from '$lib/data/fiat-currencies.json';
	import nativeCryptos from '$lib/data/native-cryptos.json';
	import cryptoTokens from '$lib/data/crypto-tokens.json';

	let items = $state<Currency[]>([]);
	let loading = $state(true);
	let hasMore = $state(false);
	let nextCursor = $state<string | undefined>();

	// Add currency dialog
	let dialogOpen = $state(false);
	let activeTab = $state('fiat');
	let fiatSearch = $state('');
	let fiatLoading = $state<string | null>(null);

	// Crypto sub-tabs
	let cryptoSubTab = $state('popular');
	let cryptoSearch = $state('');
	let cryptoLoading = $state<string | null>(null);

	// Import token list
	interface ImportToken {
		code: string;
		name: string;
		symbol: string;
		asset_scale: number;
		caip19_id: string;
		logo_uri?: string;
		selected: boolean;
	}
	let importTokens = $state<ImportToken[]>([]);
	let importListName = $state('');
	let importUrl = $state('');
	let importFetching = $state(false);
	let importAdding = $state(false);
	let importSearch = $state('');

	// Custom form
	let customForm = $state<CreateCurrencyRequest>({
		code: '',
		name: '',
		symbol: '',
		asset_scale: 18,
		asset_type: 'crypto',
		caip19_id: ''
	});
	let customLoading = $state(false);

	function openDialog() {
		activeTab = 'fiat';
		fiatSearch = '';
		cryptoSearch = '';
		cryptoSubTab = 'popular';
		dialogOpen = true;
	}

	const filteredFiat = $derived(
		fiatCurrencies.filter(
			(c) =>
				c.code.toLowerCase().includes(fiatSearch.toLowerCase()) ||
				c.name.toLowerCase().includes(fiatSearch.toLowerCase())
		)
	);

	const allCryptos = [...nativeCryptos, ...cryptoTokens];

	// Logo lookup: crypto by caip19_id, fiat by country flag
	const cryptoLogoMap = new Map<string, string>();
	for (const t of allCryptos) {
		if (t.logo_uri) cryptoLogoMap.set(t.caip19_id, t.logo_uri);
	}

	function getCurrencyLogo(currency: Currency): string | null {
		if (currency.asset_type === 'fiat') {
			const country = currency.code.slice(0, 2).toLowerCase();
			return `https://flagcdn.com/w40/${country}.png`;
		}
		return cryptoLogoMap.get(currency.caip19_id) ?? null;
	}

	const filteredCrypto = $derived(
		allCryptos.filter(
			(c) =>
				c.code.toLowerCase().includes(cryptoSearch.toLowerCase()) ||
				c.name.toLowerCase().includes(cryptoSearch.toLowerCase())
		)
	);

	const filteredImport = $derived(
		importTokens.filter(
			(c) =>
				c.code.toLowerCase().includes(importSearch.toLowerCase()) ||
				c.name.toLowerCase().includes(importSearch.toLowerCase())
		)
	);

	const selectedImportCount = $derived(importTokens.filter((t) => t.selected).length);

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

	async function addCryptoToken(token: (typeof allCryptos)[number]) {
		cryptoLoading = token.caip19_id;
		try {
			await currencies.create({
				code: token.code,
				name: token.name,
				symbol: token.symbol,
				asset_scale: token.asset_scale,
				asset_type: 'crypto',
				caip19_id: token.caip19_id
			});
			toast.success(`${token.code} added`);
			await loadCurrencies();
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to add currency');
		} finally {
			cryptoLoading = null;
		}
	}

	function parseTokenList(json: string) {
		try {
			const data = JSON.parse(json);
			if (!data.tokens || !Array.isArray(data.tokens)) {
				toast.error('Invalid token list: missing "tokens" array');
				return;
			}
			importListName = data.name || 'Unknown';
			importTokens = data.tokens
				.filter((t: Record<string, unknown>) => t.chainId && t.address && t.symbol)
				.map((t: Record<string, unknown>) => ({
					code: t.symbol as string,
					name: t.name as string,
					symbol: t.symbol as string,
					asset_scale: (t.decimals as number) ?? 18,
					caip19_id: `eip155:${t.chainId}/erc20:${t.address}`,
					logo_uri: (t.logoURI as string) || undefined,
					selected: true
				}));
			importSearch = '';
			if (importTokens.length === 0) {
				toast.error('No valid tokens found in list');
			}
		} catch {
			toast.error('Failed to parse token list JSON');
		}
	}

	async function fetchTokenList() {
		if (!importUrl.trim()) return;
		importFetching = true;
		try {
			const res = await fetch(importUrl.trim());
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const json = await res.text();
			parseTokenList(json);
		} catch {
			toast.error('Failed to fetch token list from URL');
		} finally {
			importFetching = false;
		}
	}

	function handleFileUpload(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		const reader = new FileReader();
		reader.onload = () => parseTokenList(reader.result as string);
		reader.readAsText(file);
		input.value = '';
	}

	function toggleAllImport(selected: boolean) {
		importTokens = importTokens.map((t) => ({ ...t, selected }));
	}

	async function addSelectedImportTokens() {
		const selected = importTokens.filter((t) => t.selected);
		if (selected.length === 0) return;
		importAdding = true;
		let added = 0;
		let failed = 0;
		for (const token of selected) {
			try {
				await currencies.create({
					code: token.code,
					name: token.name,
					symbol: token.symbol,
					asset_scale: token.asset_scale,
					asset_type: 'crypto',
					caip19_id: token.caip19_id
				});
				added++;
			} catch {
				failed++;
			}
		}
		if (added > 0) toast.success(`Added ${added} token${added > 1 ? 's' : ''}`);
		if (failed > 0) toast.error(`${failed} token${failed > 1 ? 's' : ''} failed (may already exist)`);
		importTokens = [];
		importListName = '';
		importUrl = '';
		await loadCurrencies();
		importAdding = false;
	}

	async function addCustom(e: Event) {
		e.preventDefault();
		customLoading = true;
		try {
			await currencies.create(customForm);
			toast.success(`${customForm.code} added`);
			dialogOpen = false;
			customForm = { code: '', name: '', symbol: '', asset_scale: 18, asset_type: 'crypto', caip19_id: '' };
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
		<Button onclick={openDialog}>
			<Plus class="mr-2 h-4 w-4" />
			Add Currency
		</Button>
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
									<a href="/currencies/{currency.id}" class="flex items-center gap-2 font-medium hover:underline">
										{#if getCurrencyLogo(currency)}
											<img
												src={getCurrencyLogo(currency)}
												alt={currency.code}
												class="h-5 w-5 rounded-full shrink-0 object-cover"
												onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
											/>
										{/if}
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

<!-- Add Currency Dialog -->
<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="max-h-[85vh] sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Add Currency</Dialog.Title>
			<Dialog.Description>Add a fiat currency or cryptocurrency.</Dialog.Description>
		</Dialog.Header>
		<Tabs.Root bind:value={activeTab}>
			<Tabs.List class="w-full">
				<Tabs.Trigger value="fiat" class="flex-1">Fiat</Tabs.Trigger>
				<Tabs.Trigger value="crypto" class="flex-1">Crypto</Tabs.Trigger>
			</Tabs.List>

			<!-- Fiat Tab -->
			<Tabs.Content value="fiat">
				<div class="space-y-4 pt-2">
					<div class="relative">
						<Search class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
						<Input placeholder="Search currencies..." bind:value={fiatSearch} class="pl-9" />
					</div>
					<div class="max-h-72 overflow-y-auto space-y-1">
						{#each filteredFiat.slice(0, 50) as fiat}
							<button
								class="flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm hover:bg-muted"
								onclick={() => addFiat(fiat)}
								disabled={fiatLoading === fiat.code}
							>
								<div class="flex items-center gap-2">
									<img
										src="https://flagcdn.com/w40/{fiat.code.slice(0, 2).toLowerCase()}.png"
										alt={fiat.code}
										class="h-5 w-5 rounded-full shrink-0 object-cover"
										onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
									/>
									<span class="font-medium">{fiat.code}</span>
									<span class="text-muted-foreground"> - {fiat.name}</span>
								</div>
								<span class="text-muted-foreground">{fiat.symbol}</span>
							</button>
						{/each}
					</div>
				</div>
			</Tabs.Content>

			<!-- Crypto Tab -->
			<Tabs.Content value="crypto">
				<Tabs.Root bind:value={cryptoSubTab}>
					<Tabs.List class="w-full mt-2">
						<Tabs.Trigger value="popular" class="flex-1 text-xs">Popular</Tabs.Trigger>
						<Tabs.Trigger value="import" class="flex-1 text-xs">Import List</Tabs.Trigger>
						<Tabs.Trigger value="custom" class="flex-1 text-xs">Custom</Tabs.Trigger>
					</Tabs.List>

					<!-- Popular Tokens -->
					<Tabs.Content value="popular">
						<div class="space-y-4 pt-2">
							<div class="relative">
								<Search class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
								<Input placeholder="Search tokens..." bind:value={cryptoSearch} class="pl-9" />
							</div>
							<div class="max-h-64 overflow-y-auto space-y-1">
								{#each filteredCrypto.slice(0, 50) as token}
									<button
										class="flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm hover:bg-muted"
										onclick={() => addCryptoToken(token)}
										disabled={cryptoLoading === token.caip19_id}
									>
										<div class="flex items-center gap-2 min-w-0">
											{#if token.logo_uri}
												<img
													src={token.logo_uri}
													alt={token.code}
													class="h-5 w-5 rounded-full shrink-0"
													onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
												/>
											{/if}
											<span class="font-medium">{token.code}</span>
											<span class="text-muted-foreground truncate"> - {token.name}</span>
										</div>
										<span class="text-muted-foreground text-xs shrink-0 ml-2">{token.asset_scale}d</span>
									</button>
								{/each}
							</div>
						</div>
					</Tabs.Content>

					<!-- Import Token List -->
					<Tabs.Content value="import">
						<div class="space-y-4 pt-2">
							{#if importTokens.length === 0}
								<p class="text-sm text-muted-foreground">
									Import tokens from a <a href="https://tokenlists.org" target="_blank" rel="noopener noreferrer" class="underline">Uniswap Token List</a> JSON file or URL.
								</p>
								<div class="space-y-3">
									<div class="space-y-2">
										<Label for="import-url">Token list URL</Label>
										<div class="flex gap-2">
											<Input
												id="import-url"
												bind:value={importUrl}
												placeholder="https://tokens.uniswap.org"
											/>
											<Button
												variant="outline"
												onclick={fetchTokenList}
												disabled={importFetching || !importUrl.trim()}
											>
												{importFetching ? 'Fetching...' : 'Fetch'}
											</Button>
										</div>
									</div>
									<div class="relative flex items-center">
										<div class="flex-grow border-t border-muted"></div>
										<span class="mx-3 text-xs text-muted-foreground">or</span>
										<div class="flex-grow border-t border-muted"></div>
									</div>
									<div>
										<input
											type="file"
											accept=".json,application/json"
											onchange={handleFileUpload}
											class="hidden"
											id="token-file-input"
										/>
										<Button
											variant="outline"
											class="w-full"
											onclick={() => document.getElementById('token-file-input')?.click()}
										>
											<Upload class="mr-2 h-4 w-4" />
											Upload JSON File
										</Button>
									</div>
								</div>
							{:else}
								<div class="flex items-center justify-between">
									<p class="text-sm font-medium">{importListName}</p>
									<p class="text-xs text-muted-foreground">{selectedImportCount} of {importTokens.length} selected</p>
								</div>
								<div class="relative">
									<Search class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
									<Input placeholder="Search tokens..." bind:value={importSearch} class="pl-9" />
								</div>
								<div class="flex gap-2">
									<Button variant="outline" size="sm" onclick={() => toggleAllImport(true)}>Select All</Button>
									<Button variant="outline" size="sm" onclick={() => toggleAllImport(false)}>Deselect All</Button>
								</div>
								<div class="max-h-48 overflow-y-auto space-y-1">
									{#each filteredImport as token}
										<button
											class="flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm hover:bg-muted"
											onclick={() => { token.selected = !token.selected; }}
										>
											<div class="flex items-center gap-2 min-w-0">
												<div class="flex h-4 w-4 shrink-0 items-center justify-center rounded border {token.selected ? 'bg-primary border-primary' : 'border-muted-foreground/30'}">
													{#if token.selected}
														<Check class="h-3 w-3 text-primary-foreground" />
													{/if}
												</div>
												{#if token.logo_uri}
													<img
														src={token.logo_uri}
														alt={token.code}
														class="h-5 w-5 rounded-full shrink-0"
														onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
													/>
												{/if}
												<span class="font-medium">{token.code}</span>
												<span class="text-muted-foreground truncate"> - {token.name}</span>
											</div>
											<span class="text-xs text-muted-foreground font-mono shrink-0 ml-2">{token.asset_scale}d</span>
										</button>
									{/each}
								</div>
								<div class="flex justify-end gap-2">
									<Button variant="outline" onclick={() => { importTokens = []; importListName = ''; importUrl = ''; }}>
										Back
									</Button>
									<Button
										onclick={addSelectedImportTokens}
										disabled={importAdding || selectedImportCount === 0}
									>
										{importAdding ? 'Adding...' : `Add ${selectedImportCount} Token${selectedImportCount !== 1 ? 's' : ''}`}
									</Button>
								</div>
							{/if}
						</div>
					</Tabs.Content>

					<!-- Custom -->
					<Tabs.Content value="custom">
						<form onsubmit={addCustom} class="space-y-4 pt-2">
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
								<Button variant="outline" type="button" onclick={() => (dialogOpen = false)}>
									Cancel
								</Button>
								<Button type="submit" disabled={customLoading}>
									{customLoading ? 'Adding...' : 'Add Currency'}
								</Button>
							</div>
						</form>
					</Tabs.Content>
				</Tabs.Root>
			</Tabs.Content>
		</Tabs.Root>
	</Dialog.Content>
</Dialog.Root>
