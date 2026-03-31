"use client";

import { useCallback, useEffect, useState } from "react";
import { api } from "@/lib/api-client";

interface User {
  id: string;
  email: string;
  full_name: string;
  role: string;
  has_active_subscription: boolean;
}

export function useAuth() {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchUser = useCallback(async () => {
    try {
      const me = await api.getMe();
      setUser(me);
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchUser();
  }, [fetchUser]);

  const login = async (email: string, password: string) => {
    await api.login({ email, password });
    await fetchUser();
  };

  const logout = async () => {
    await api.logout();
    setUser(null);
  };

  return {
    user,
    loading,
    isAuthenticated: !!user,
    hasSubscription: user?.has_active_subscription ?? false,
    login,
    logout,
    refresh: fetchUser,
  };
}
