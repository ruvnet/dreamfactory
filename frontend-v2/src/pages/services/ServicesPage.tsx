import React, { useState } from 'react';
import { PlusIcon, Cog6ToothIcon, TrashIcon, EyeIcon } from '@heroicons/react/24/outline';
import { Button } from '../../components/atoms/Button';
import { Input } from '../../components/atoms/Input';
import { Badge } from '../../components/atoms/Badge';
import { Card } from '../../components/molecules/Card';
import { Modal } from '../../components/molecules/Modal';
import { DataTable } from '../../components/organisms/DataTable';
import { ServiceForm } from '../../components/organisms/ServiceForm';
import { useServices, useCreateService, useUpdateService, useDeleteService } from '../../lib/api/services/services';

const ServiceStatusBadge: React.FC<{ status: string }> = ({ status }) => {
  const variant = status === 'Active' ? 'success' : status === 'Inactive' ? 'error' : 'warning';
  return <Badge variant={variant}>{status}</Badge>;
};

const ServiceTypeBadge: React.FC<{ type: string }> = ({ type }) => {
  const getTypeColor = (type: string) => {
    switch (type.toLowerCase()) {
      case 'database': return 'blue';
      case 'file': return 'green';
      case 'email': return 'purple';
      case 'remote_web': return 'orange';
      case 'script': return 'indigo';
      default: return 'gray';
    }
  };
  
  return <Badge variant={getTypeColor(type)}>{type}</Badge>;
};

export const ServicesPage: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [selectedService, setSelectedService] = useState<any>(null);
  
  const { data: servicesResponse, isLoading, error } = useServices();
  const createService = useCreateService();
  const updateService = useUpdateService();
  const deleteService = useDeleteService();
  
  const services = servicesResponse?.resource || [];

  const columns = [
    {
      key: 'name',
      header: 'Service Name',
      sortable: true,
      render: (service: any) => (
        <div className="flex items-center space-x-3">
          <div className="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-lg flex items-center justify-center">
            <Cog6ToothIcon className="w-4 h-4 text-blue-600" />
          </div>
          <div>
            <div className="font-medium text-gray-900">{service.name}</div>
            <div className="text-sm text-gray-500">{service.description}</div>
          </div>
        </div>
      ),
    },
    {
      key: 'type',
      header: 'Type',
      sortable: true,
      render: (service: any) => <ServiceTypeBadge type={service.type} />,
    },
    {
      key: 'status',
      header: 'Status',
      sortable: true,
      render: (service: any) => <ServiceStatusBadge status={service.is_active ? 'Active' : 'Inactive'} />,
    },
    {
      key: 'base_url',
      header: 'Base URL',
      render: (service: any) => (
        <code className="text-sm bg-gray-100 px-2 py-1 rounded">
          /api/v2/{service.name}
        </code>
      ),
    },
    {
      key: 'created_date',
      header: 'Created',
      sortable: true,
      render: (service: any) => new Date(service.created_date).toLocaleDateString(),
    },
    {
      key: 'actions',
      header: 'Actions',
      render: (service: any) => (
        <div className="flex space-x-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleViewService(service)}
            className="text-blue-600 hover:text-blue-700"
          >
            <EyeIcon className="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleEditService(service)}
            className="text-gray-600 hover:text-gray-700"
          >
            <Cog6ToothIcon className="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => handleDeleteService(service)}
            className="text-red-600 hover:text-red-700"
          >
            <TrashIcon className="w-4 h-4" />
          </Button>
        </div>
      ),
    },
  ];

  const handleCreateService = async (serviceData: any) => {
    try {
      await createService.mutateAsync(serviceData);
      setIsCreateModalOpen(false);
    } catch (error) {
      console.error('Failed to create service:', error);
    }
  };

  const handleEditService = (service: any) => {
    setSelectedService(service);
    setIsEditModalOpen(true);
  };

  const handleUpdateService = async (serviceData: any) => {
    try {
      await updateService.mutateAsync({ id: selectedService.id, data: serviceData });
      setIsEditModalOpen(false);
      setSelectedService(null);
    } catch (error) {
      console.error('Failed to update service:', error);
    }
  };

  const handleDeleteService = async (service: any) => {
    if (window.confirm(`Are you sure you want to delete the service "${service.name}"?`)) {
      try {
        await deleteService.mutateAsync(service.id);
      } catch (error) {
        console.error('Failed to delete service:', error);
      }
    }
  };

  const handleViewService = (service: any) => {
    // Navigate to service details page
    window.open(`/api/v2/${service.name}`, '_blank');
  };

  const filteredServices = services?.filter(service =>
    service.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    service.type.toLowerCase().includes(searchTerm.toLowerCase()) ||
    service.description?.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  if (error) {
    return (
      <div className="p-6">
        <Card className="p-8 text-center">
          <div className="text-red-600 text-lg font-medium mb-2">Error loading services</div>
          <div className="text-gray-600">{error.message}</div>
        </Card>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Services</h1>
          <p className="text-gray-600 mt-1">
            Manage API services and data connections
          </p>
        </div>
        <Button
          variant="primary"
          onClick={() => setIsCreateModalOpen(true)}
          className="flex items-center space-x-2"
        >
          <PlusIcon className="w-4 h-4" />
          <span>Create Service</span>
        </Button>
      </div>

      {/* Search and Filters */}
      <Card className="p-4">
        <div className="flex space-x-4">
          <div className="flex-1">
            <Input
              type="text"
              placeholder="Search services..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full"
            />
          </div>
        </div>
      </Card>

      {/* Services Table */}
      <Card>
        <DataTable
          data={filteredServices}
          columns={columns}
          loading={isLoading}
          emptyMessage="No services found. Create your first service to get started."
        />
      </Card>

      {/* Create Service Modal */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title="Create New Service"
        size="lg"
      >
        <ServiceForm
          onSubmit={handleCreateService}
          onCancel={() => setIsCreateModalOpen(false)}
          isSubmitting={createService.isPending}
        />
      </Modal>

      {/* Edit Service Modal */}
      <Modal
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        title="Edit Service"
        size="lg"
      >
        {selectedService && (
          <ServiceForm
            initialData={selectedService}
            onSubmit={handleUpdateService}
            onCancel={() => setIsEditModalOpen(false)}
            isSubmitting={updateService.isPending}
          />
        )}
      </Modal>
    </div>
  );
};

export default ServicesPage;