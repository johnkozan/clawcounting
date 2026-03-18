import { writable, derived } from 'svelte/store';
import { auth as authApi, type User } from '$lib/api';

function createAuthStore() {
	const user = writable<User | null>(null);
	const loading = writable(true);
	const needsSetup = writable(false);
	const isAuthenticated = derived(user, ($user) => $user !== null);

	async function initialize() {
		// Check if setup is needed (no users exist yet)
		try {
			console.log('[auth] checking setup status...');
			const status = await authApi.setupStatus();
			console.log('[auth] setup status response:', status);
			if (status.needs_setup) {
				needsSetup.set(true);
				loading.set(false);
				console.log('[auth] needs setup, returning early');
				return;
			}
		} catch (err) {
			console.error('[auth] setup status check failed:', err);
			// If check fails, continue with normal auth flow
		}

		const token = localStorage.getItem('token');
		console.log('[auth] token present:', !!token);
		if (!token) {
			loading.set(false);
			return;
		}
		try {
			const res = await authApi.me();
			user.set(res.data);
		} catch {
			localStorage.removeItem('token');
			localStorage.removeItem('refresh_token');
		} finally {
			loading.set(false);
		}
	}

	async function login(email: string, password: string) {
		const res = await authApi.login(email, password);
		localStorage.setItem('token', res.access_token);
		localStorage.setItem('refresh_token', res.refresh_token);
		const meRes = await authApi.me();
		user.set(meRes.data);
		needsSetup.set(false);
	}

	function logout() {
		localStorage.removeItem('token');
		localStorage.removeItem('refresh_token');
		user.set(null);
	}

	return {
		user,
		loading,
		needsSetup,
		isAuthenticated,
		initialize,
		login,
		logout
	};
}

export const authStore = createAuthStore();
