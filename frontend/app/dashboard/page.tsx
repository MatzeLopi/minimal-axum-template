"use client";
import { useEffect, useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { useRouter } from 'next/navigation';
import api from '../lib/api';
import { Lock, LogOut, User } from 'lucide-react'; // Make sure to install lucide-react

export default function Dashboard() {
    const { user, logout, isLoading } = useAuth();
    const router = useRouter();

    // Password Change State
    const [pwData, setPwData] = useState({ old_password: '', new_password: '' });
    const [status, setStatus] = useState({ type: '', msg: '' });

    useEffect(() => {
        if (!isLoading && !user) router.push('/login');
    }, [user, isLoading, router]);

    const handlePasswordChange = async (e: React.FormEvent) => {
        e.preventDefault();
        setStatus({ type: '', msg: '' });

        // Frontend Validation (mirroring backend requirements)
        if (pwData.new_password.length < 8) {
            setStatus({ type: 'error', msg: 'New password must be at least 8 chars' });
            return;
        }

        try {
            // Matches the DTO we defined: { old_password, new_password }
            await api.post('/users/password', pwData);
            setStatus({ type: 'success', msg: 'Password updated successfully!' });
            setPwData({ old_password: '', new_password: '' });
        } catch (err) {
            setStatus({ type: 'error', msg: 'Failed to update password. Check old password.' });
        }
    };

    if (isLoading || !user) return <div className="p-10">Loading...</div>;

    return (
        <div className="min-h-screen bg-gray-50 p-8">
            <div className="mx-auto max-w-4xl">
                {/* Header */}
                <div className="mb-8 flex items-center justify-between rounded-xl bg-white p-6 shadow-sm">
                    <div className="flex items-center gap-4">
                        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-indigo-100 text-indigo-600">
                            <User size={24} />
                        </div>
                        <div>
                            <h1 className="text-xl font-bold text-gray-900">Welcome, {user.username}</h1>
                            <p className="text-sm text-gray-500">{user.email}</p>
                        </div>
                    </div>
                    <button onClick={logout} className="flex items-center gap-2 rounded-lg border border-gray-200 px-4 py-2 text-sm font-medium hover:bg-gray-50">
                        <LogOut size={16} /> Logout
                    </button>
                </div>

                {/* Security Section */}
                <div className="rounded-xl bg-white p-6 shadow-sm">
                    <div className="mb-6 flex items-center gap-2 border-b pb-4">
                        <Lock size={20} className="text-gray-400" />
                        <h2 className="text-lg font-semibold">Security Settings</h2>
                    </div>

                    <form onSubmit={handlePasswordChange} className="max-w-md space-y-4">
                        {status.msg && (
                            <div className={`p-3 rounded text-sm ${status.type === 'error' ? 'bg-red-50 text-red-600' : 'bg-green-50 text-green-600'}`}>
                                {status.msg}
                            </div>
                        )}

                        <div>
                            <label className="mb-1 block text-sm font-medium text-gray-700">Current Password</label>
                            <input
                                type="password"
                                value={pwData.old_password}
                                onChange={(e) => setPwData({ ...pwData, old_password: e.target.value })}
                                className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-indigo-500 focus:outline-none"
                            />
                        </div>

                        <div>
                            <label className="mb-1 block text-sm font-medium text-gray-700">New Password</label>
                            <input
                                type="password"
                                value={pwData.new_password}
                                onChange={(e) => setPwData({ ...pwData, new_password: e.target.value })}
                                className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:border-indigo-500 focus:outline-none"
                            />
                            <p className="mt-1 text-xs text-gray-500">Minimum 8 characters</p>
                        </div>

                        <button type="submit" className="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700">
                            Update Password
                        </button>
                    </form>
                </div>
            </div>
        </div>
    );
}