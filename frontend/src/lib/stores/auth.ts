import { writable, derived } from 'svelte/store';
import { auth as authApi, type User } from '$lib/api';

function createAuthStore() {
	const user = writable<User | null>(null);
	const loading = writable(true);
	const isAuthenticated = derived(user, ($user) => $user !== null);

	async function initialize() {
		const token = localStorage.getItem('token');
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
	}

	function logout() {
		localStorage.removeItem('token');
		localStorage.removeItem('refresh_token');
		user.set(null);
	}

	return {
		user,
		loading,
		isAuthenticated,
		initialize,
		login,
		logout
	};
}

export const authStore = createAuthStore();
