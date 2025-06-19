import { useState, useEffect, Fragment } from 'react';
import { Outlet, Link, useLocation, useNavigate } from 'react-router-dom';
import { Dialog, Transition } from '@headlessui/react';
import {
  Bars3Icon,
  XMarkIcon,
  HomeIcon,
  UsersIcon,
  CubeIcon,
  DocumentTextIcon,
  CogIcon,
  ArrowRightOnRectangleIcon,
  UserCircleIcon,
  ChevronDownIcon,
} from '@heroicons/react/24/outline';
import { useUIStore } from '../stores/ui.store';
import { useCurrentUser } from '../lib/api/services/auth';
import { useLogout } from '../lib/api/services/auth';
import { cn } from '../utils/cn';

interface NavigationItem {
  name: string;
  href: string;
  icon: React.ComponentType<{ className?: string }>;
  children?: NavigationItem[];
}

const navigation: NavigationItem[] = [
  { name: 'Dashboard', href: '/', icon: HomeIcon },
  { 
    name: 'Users & Roles', 
    href: '/users', 
    icon: UsersIcon,
    children: [
      { name: 'Users', href: '/users', icon: UsersIcon },
      { name: 'Roles', href: '/roles', icon: UserCircleIcon },
      { name: 'Admins', href: '/admins', icon: UserCircleIcon },
    ]
  },
  { 
    name: 'Services', 
    href: '/services', 
    icon: CubeIcon,
    children: [
      { name: 'All Services', href: '/services', icon: CubeIcon },
      { name: 'Create Service', href: '/services/create', icon: CubeIcon },
      { name: 'Service Types', href: '/service-types', icon: CubeIcon },
    ]
  },
  { 
    name: 'Applications', 
    href: '/apps', 
    icon: DocumentTextIcon,
    children: [
      { name: 'All Apps', href: '/apps', icon: DocumentTextIcon },
      { name: 'Create App', href: '/apps/create', icon: DocumentTextIcon },
      { name: 'API Keys', href: '/api-keys', icon: DocumentTextIcon },
    ]
  },
  { 
    name: 'System', 
    href: '/system', 
    icon: CogIcon,
    children: [
      { name: 'Configuration', href: '/system/config', icon: CogIcon },
      { name: 'Email Templates', href: '/system/email-templates', icon: CogIcon },
      { name: 'Events', href: '/system/events', icon: CogIcon },
      { name: 'Scheduler', href: '/system/scheduler', icon: CogIcon },
      { name: 'Cache', href: '/system/cache', icon: CogIcon },
      { name: 'Logs', href: '/system/logs', icon: CogIcon },
    ]
  },
];

export function MainLayout() {
  const location = useLocation();
  const navigate = useNavigate();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [expandedItems, setExpandedItems] = useState<string[]>([]);
  
  const { theme, setTheme, sidebarOpen: uiSidebarOpen, setSidebarOpen: setUISidebarOpen } = useUIStore();
  const { user: currentUser } = useCurrentUser();
  const logout = useLogout();
  
  // Auto-expand parent items based on current route
  useEffect(() => {
    const path = location.pathname;
    const expanded: string[] = [];
    
    navigation.forEach(item => {
      if (item.children) {
        const hasActiveChild = item.children.some(child => path.startsWith(child.href));
        if (hasActiveChild) {
          expanded.push(item.name);
        }
      }
    });
    
    setExpandedItems(expanded);
  }, [location.pathname]);
  
  const toggleExpanded = (itemName: string) => {
    setExpandedItems(prev => 
      prev.includes(itemName) 
        ? prev.filter(name => name !== itemName)
        : [...prev, itemName]
    );
  };
  
  const handleLogout = async () => {
    try {
      await logout.mutateAsync({ isSysAdmin: currentUser?.isSysAdmin });
      navigate('/auth/login');
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };
  
  const isActive = (href: string) => {
    if (href === '/') {
      return location.pathname === '/';
    }
    return location.pathname.startsWith(href);
  };
  
  return (
    <div className="h-screen flex overflow-hidden bg-gray-100 dark:bg-gray-900">
      {/* Mobile sidebar */}
      <Transition.Root show={sidebarOpen} as={Fragment}>
        <Dialog as="div" className="relative z-40 md:hidden" onClose={setSidebarOpen}>
          <Transition.Child
            as={Fragment}
            enter="transition-opacity ease-linear duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity ease-linear duration-300"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-gray-600 bg-opacity-75" />
          </Transition.Child>

          <div className="fixed inset-0 z-40 flex">
            <Transition.Child
              as={Fragment}
              enter="transition ease-in-out duration-300 transform"
              enterFrom="-translate-x-full"
              enterTo="translate-x-0"
              leave="transition ease-in-out duration-300 transform"
              leaveFrom="translate-x-0"
              leaveTo="-translate-x-full"
            >
              <Dialog.Panel className="relative flex-1 flex flex-col max-w-xs w-full bg-white dark:bg-gray-800">
                <Transition.Child
                  as={Fragment}
                  enter="ease-in-out duration-300"
                  enterFrom="opacity-0"
                  enterTo="opacity-100"
                  leave="ease-in-out duration-300"
                  leaveFrom="opacity-100"
                  leaveTo="opacity-0"
                >
                  <div className="absolute top-0 right-0 -mr-12 pt-2">
                    <button
                      type="button"
                      className="ml-1 flex items-center justify-center h-10 w-10 rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                      onClick={() => setSidebarOpen(false)}
                    >
                      <span className="sr-only">Close sidebar</span>
                      <XMarkIcon className="h-6 w-6 text-white" aria-hidden="true" />
                    </button>
                  </div>
                </Transition.Child>
                
                <div className="flex-1 h-0 pt-5 pb-4 overflow-y-auto">
                  <div className="flex-shrink-0 flex items-center px-4">
                    <img
                      className="h-8 w-auto"
                      src="/dreamfactory-logo.svg"
                      alt="DreamFactory"
                    />
                  </div>
                  <nav className="mt-5 px-2 space-y-1">
                    {navigation.map((item) => (
                      <NavigationItem
                        key={item.name}
                        item={item}
                        isActive={isActive}
                        isExpanded={expandedItems.includes(item.name)}
                        onToggle={() => toggleExpanded(item.name)}
                      />
                    ))}
                  </nav>
                </div>
                
                <div className="flex-shrink-0 flex border-t border-gray-200 dark:border-gray-700 p-4">
                  <a href="#" className="flex-shrink-0 group block">
                    <div className="flex items-center">
                      <div>
                        <UserCircleIcon className="inline-block h-10 w-10 rounded-full text-gray-400" />
                      </div>
                      <div className="ml-3">
                        <p className="text-base font-medium text-gray-700 dark:text-gray-200">
                          {currentUser?.name || 'User'}
                        </p>
                        <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
                          View profile
                        </p>
                      </div>
                    </div>
                  </a>
                </div>
              </Dialog.Panel>
            </Transition.Child>
            
            <div className="flex-shrink-0 w-14" aria-hidden="true">
              {/* Force sidebar to shrink to fit close icon */}
            </div>
          </div>
        </Dialog>
      </Transition.Root>

      {/* Static sidebar for desktop */}
      <div className="hidden md:flex md:w-64 md:flex-col md:fixed md:inset-y-0">
        <div className="flex-1 flex flex-col min-h-0 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
          <div className="flex-1 flex flex-col pt-5 pb-4 overflow-y-auto">
            <div className="flex items-center flex-shrink-0 px-4">
              <img
                className="h-8 w-auto"
                src="/dreamfactory-logo.svg"
                alt="DreamFactory"
              />
            </div>
            <nav className="mt-5 flex-1 px-2 space-y-1">
              {navigation.map((item) => (
                <NavigationItem
                  key={item.name}
                  item={item}
                  isActive={isActive}
                  isExpanded={expandedItems.includes(item.name)}
                  onToggle={() => toggleExpanded(item.name)}
                />
              ))}
            </nav>
          </div>
          
          <div className="flex-shrink-0 flex border-t border-gray-200 dark:border-gray-700 p-4">
            <a href="#" className="flex-shrink-0 w-full group block">
              <div className="flex items-center">
                <div>
                  <UserCircleIcon className="inline-block h-9 w-9 rounded-full text-gray-400" />
                </div>
                <div className="ml-3">
                  <p className="text-sm font-medium text-gray-700 dark:text-gray-200">
                    {currentUser?.name || 'User'}
                  </p>
                  <p className="text-xs font-medium text-gray-500 dark:text-gray-400">
                    {currentUser?.email}
                  </p>
                </div>
              </div>
            </a>
          </div>
        </div>
      </div>
      
      <div className="md:pl-64 flex flex-col flex-1">
        <div className="sticky top-0 z-10 md:hidden pl-1 pt-1 sm:pl-3 sm:pt-3 bg-gray-100 dark:bg-gray-900">
          <button
            type="button"
            className="-ml-0.5 -mt-0.5 h-12 w-12 inline-flex items-center justify-center rounded-md text-gray-500 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500"
            onClick={() => setSidebarOpen(true)}
          >
            <span className="sr-only">Open sidebar</span>
            <Bars3Icon className="h-6 w-6" aria-hidden="true" />
          </button>
        </div>
        
        <header className="bg-white dark:bg-gray-800 shadow-sm">
          <div className="px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between h-16">
              <div className="flex items-center">
                <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
                  {getPageTitle(location.pathname)}
                </h1>
              </div>
              
              <div className="flex items-center space-x-4">
                <button
                  onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
                  className="p-2 text-gray-400 hover:text-gray-500 dark:hover:text-gray-300"
                >
                  {theme === 'light' ? '🌙' : '☀️'}
                </button>
                
                <button
                  onClick={handleLogout}
                  className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                >
                  <ArrowRightOnRectangleIcon className="h-4 w-4 mr-2" />
                  Logout
                </button>
              </div>
            </div>
          </div>
        </header>
        
        <main className="flex-1 overflow-y-auto">
          <div className="py-6">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
              <Outlet />
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}

// Navigation Item Component
interface NavigationItemProps {
  item: NavigationItem;
  isActive: (href: string) => boolean;
  isExpanded: boolean;
  onToggle: () => void;
}

function NavigationItem({ item, isActive, isExpanded, onToggle }: NavigationItemProps) {
  const hasChildren = item.children && item.children.length > 0;
  
  if (hasChildren) {
    return (
      <div>
        <button
          onClick={onToggle}
          className={cn(
            'group flex items-center justify-between w-full px-2 py-2 text-sm font-medium rounded-md',
            isActive(item.href)
              ? 'bg-gray-100 text-gray-900 dark:bg-gray-900 dark:text-white'
              : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white'
          )}
        >
          <div className="flex items-center">
            <item.icon
              className={cn(
                'mr-3 flex-shrink-0 h-6 w-6',
                isActive(item.href)
                  ? 'text-gray-500 dark:text-gray-300'
                  : 'text-gray-400 group-hover:text-gray-500 dark:text-gray-400 dark:group-hover:text-gray-300'
              )}
              aria-hidden="true"
            />
            {item.name}
          </div>
          <ChevronDownIcon
            className={cn(
              'ml-auto h-5 w-5 transform transition-transform',
              isExpanded ? 'rotate-180' : ''
            )}
          />
        </button>
        
        {isExpanded && (
          <div className="space-y-1 mt-1">
            {item.children.map((child) => (
              <Link
                key={child.name}
                to={child.href}
                className={cn(
                  'group flex items-center pl-11 pr-2 py-2 text-sm font-medium rounded-md',
                  isActive(child.href)
                    ? 'bg-gray-100 text-gray-900 dark:bg-gray-900 dark:text-white'
                    : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white'
                )}
              >
                {child.name}
              </Link>
            ))}
          </div>
        )}
      </div>
    );
  }
  
  return (
    <Link
      to={item.href}
      className={cn(
        'group flex items-center px-2 py-2 text-sm font-medium rounded-md',
        isActive(item.href)
          ? 'bg-gray-100 text-gray-900 dark:bg-gray-900 dark:text-white'
          : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white'
      )}
    >
      <item.icon
        className={cn(
          'mr-3 flex-shrink-0 h-6 w-6',
          isActive(item.href)
            ? 'text-gray-500 dark:text-gray-300'
            : 'text-gray-400 group-hover:text-gray-500 dark:text-gray-400 dark:group-hover:text-gray-300'
        )}
        aria-hidden="true"
      />
      {item.name}
    </Link>
  );
}

// Helper function to get page title from route
function getPageTitle(pathname: string): string {
  const titles: Record<string, string> = {
    '/': 'Dashboard',
    '/users': 'User Management',
    '/roles': 'Role Management',
    '/admins': 'Admin Management',
    '/services': 'Services',
    '/services/create': 'Create Service',
    '/service-types': 'Service Types',
    '/apps': 'Applications',
    '/apps/create': 'Create Application',
    '/api-keys': 'API Keys',
    '/system/config': 'System Configuration',
    '/system/email-templates': 'Email Templates',
    '/system/events': 'Event Management',
    '/system/scheduler': 'Task Scheduler',
    '/system/cache': 'Cache Management',
    '/system/logs': 'System Logs',
  };
  
  return titles[pathname] || 'DreamFactory';
}