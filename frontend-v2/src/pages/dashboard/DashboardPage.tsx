import { useQuery } from '@tanstack/react-query';
import { 
  ChartBarIcon,
  UsersIcon, 
  CubeIcon, 
  DocumentTextIcon,
  ClockIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';
import { cn } from '../../utils/cn';

interface DashboardStats {
  totalUsers: number;
  totalServices: number;
  totalApps: number;
  activeApps: number;
  apiCalls24h: number;
  errorRate: number;
}

export default function DashboardPage() {
  // TODO: Implement actual API calls
  const { data: stats } = useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: async (): Promise<DashboardStats> => {
      // Mock data for now
      return {
        totalUsers: 156,
        totalServices: 12,
        totalApps: 8,
        activeApps: 6,
        apiCalls24h: 15420,
        errorRate: 0.8,
      };
    },
  });

  const statCards = [
    {
      name: 'Total Users',
      value: stats?.totalUsers || 0,
      icon: UsersIcon,
      change: '+12%',
      changeType: 'increase' as const,
    },
    {
      name: 'Active Services',
      value: stats?.totalServices || 0,
      icon: CubeIcon,
      change: '+2',
      changeType: 'increase' as const,
    },
    {
      name: 'Applications',
      value: stats?.totalApps || 0,
      icon: DocumentTextIcon,
      change: '8 total',
      changeType: 'neutral' as const,
    },
    {
      name: 'API Calls (24h)',
      value: stats?.apiCalls24h || 0,
      icon: ChartBarIcon,
      change: '+8.2%',
      changeType: 'increase' as const,
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="min-w-0 flex-1">
          <h2 className="text-2xl font-bold leading-7 text-gray-900 dark:text-white sm:truncate sm:text-3xl sm:tracking-tight">
            Dashboard
          </h2>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Welcome back! Here's what's happening with your DreamFactory instance.
          </p>
        </div>
        <div className="mt-4 flex md:ml-4 md:mt-0">
          <button
            type="button"
            className="inline-flex items-center rounded-md bg-white dark:bg-gray-800 px-3 py-2 text-sm font-semibold text-gray-900 dark:text-white shadow-sm ring-1 ring-inset ring-gray-300 dark:ring-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            <ClockIcon className="-ml-0.5 mr-1.5 h-5 w-5 text-gray-400" aria-hidden="true" />
            View Logs
          </button>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {statCards.map((card) => (
          <div
            key={card.name}
            className="relative overflow-hidden rounded-lg bg-white dark:bg-gray-800 px-4 py-5 shadow sm:px-6 sm:py-6"
          >
            <dt>
              <div className="absolute rounded-md bg-indigo-500 p-3">
                <card.icon className="h-6 w-6 text-white" aria-hidden="true" />
              </div>
              <p className="ml-16 truncate text-sm font-medium text-gray-500 dark:text-gray-400">
                {card.name}
              </p>
            </dt>
            <dd className="ml-16 flex items-baseline pb-6 sm:pb-7">
              <p className="text-2xl font-semibold text-gray-900 dark:text-white">
                {typeof card.value === 'number' ? card.value.toLocaleString() : card.value}
              </p>
              <p
                className={cn(
                  'ml-2 flex items-baseline text-sm font-semibold',
                  card.changeType === 'increase'
                    ? 'text-green-600 dark:text-green-400'
                    : card.changeType === 'decrease'
                    ? 'text-red-600 dark:text-red-400'
                    : 'text-gray-500 dark:text-gray-400'
                )}
              >
                {card.change}
              </p>
            </dd>
          </div>
        ))}
      </div>

      {/* Quick Actions & Recent Activity */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Quick Actions */}
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">Quick Actions</h3>
            <div className="mt-6 grid grid-cols-1 gap-4">
              <button className="flex items-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors">
                <UsersIcon className="h-8 w-8 text-indigo-600 dark:text-indigo-400" />
                <div className="ml-4 text-left">
                  <p className="text-sm font-medium text-gray-900 dark:text-white">Create User</p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">Add a new user to the system</p>
                </div>
              </button>
              
              <button className="flex items-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors">
                <CubeIcon className="h-8 w-8 text-green-600 dark:text-green-400" />
                <div className="ml-4 text-left">
                  <p className="text-sm font-medium text-gray-900 dark:text-white">New Service</p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">Connect a new data source</p>
                </div>
              </button>
              
              <button className="flex items-center p-4 bg-gray-50 dark:bg-gray-700 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors">
                <DocumentTextIcon className="h-8 w-8 text-blue-600 dark:text-blue-400" />
                <div className="ml-4 text-left">
                  <p className="text-sm font-medium text-gray-900 dark:text-white">Create App</p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">Build a new application</p>
                </div>
              </button>
            </div>
          </div>
        </div>

        {/* System Status */}
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-6">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">System Status</h3>
            <div className="mt-6 space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-500 dark:text-gray-400">API Health</span>
                <span className="inline-flex items-center rounded-full bg-green-100 dark:bg-green-900 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:text-green-200">
                  Healthy
                </span>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-500 dark:text-gray-400">Database</span>
                <span className="inline-flex items-center rounded-full bg-green-100 dark:bg-green-900 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:text-green-200">
                  Connected
                </span>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-500 dark:text-gray-400">Cache</span>
                <span className="inline-flex items-center rounded-full bg-green-100 dark:bg-green-900 px-2.5 py-0.5 text-xs font-medium text-green-800 dark:text-green-200">
                  Active
                </span>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-500 dark:text-gray-400">Error Rate</span>
                <span className="inline-flex items-center rounded-full bg-yellow-100 dark:bg-yellow-900 px-2.5 py-0.5 text-xs font-medium text-yellow-800 dark:text-yellow-200">
                  {stats?.errorRate}%
                </span>
              </div>
            </div>
            
            {stats?.errorRate && stats.errorRate > 1 && (
              <div className="mt-4 p-3 bg-yellow-50 dark:bg-yellow-900/50 rounded-md">
                <div className="flex">
                  <ExclamationTriangleIcon className="h-5 w-5 text-yellow-400" />
                  <div className="ml-3">
                    <p className="text-sm text-yellow-700 dark:text-yellow-200">
                      Error rate is elevated. Check system logs for details.
                    </p>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
        <div className="p-6">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">Recent Activity</h3>
          <div className="mt-6">
            <div className="flow-root">
              <ul role="list" className="-mb-8">
                {/* Mock activity items */}
                {[
                  { id: 1, action: 'User john.doe@example.com logged in', time: '2 minutes ago', type: 'user' },
                  { id: 2, action: 'Service "MySQL Database" connection tested', time: '15 minutes ago', type: 'service' },
                  { id: 3, action: 'New app "Mobile API" created', time: '1 hour ago', type: 'app' },
                  { id: 4, action: 'Role permissions updated for "Developers"', time: '2 hours ago', type: 'role' },
                ].map((item, itemIdx, items) => (
                  <li key={item.id}>
                    <div className="relative pb-8">
                      {itemIdx !== items.length - 1 ? (
                        <span
                          className="absolute left-4 top-4 -ml-px h-full w-0.5 bg-gray-200 dark:bg-gray-700"
                          aria-hidden="true"
                        />
                      ) : null}
                      <div className="relative flex space-x-3">
                        <div>
                          <span className={cn(
                            'h-8 w-8 rounded-full flex items-center justify-center ring-8 ring-white dark:ring-gray-800',
                            item.type === 'user' ? 'bg-blue-500' :
                            item.type === 'service' ? 'bg-green-500' :
                            item.type === 'app' ? 'bg-purple-500' : 'bg-orange-500'
                          )}>
                            {item.type === 'user' && <UsersIcon className="h-4 w-4 text-white" />}
                            {item.type === 'service' && <CubeIcon className="h-4 w-4 text-white" />}
                            {item.type === 'app' && <DocumentTextIcon className="h-4 w-4 text-white" />}
                            {item.type === 'role' && <UsersIcon className="h-4 w-4 text-white" />}
                          </span>
                        </div>
                        <div className="flex min-w-0 flex-1 justify-between space-x-4 pt-1.5">
                          <div>
                            <p className="text-sm text-gray-500 dark:text-gray-400">
                              {item.action}
                            </p>
                          </div>
                          <div className="whitespace-nowrap text-right text-sm text-gray-500 dark:text-gray-400">
                            {item.time}
                          </div>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}