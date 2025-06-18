//! Benchmarks for df-core service registry performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use df_core::prelude::*;
use df_core::service::{BaseService, ServiceInfo};
use df_core::registry::ServiceRegistry;
use tokio::runtime::Runtime;

fn bench_service_registration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("service_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let registry = ServiceRegistry::new();
                
                for i in 0..black_box(100) {
                    let info = ServiceInfo::new(
                        format!("service-{}", i),
                        "1.0.0",
                        format!("Benchmark service {}", i)
                    );
                    let service = BaseService::new(info);
                    registry.register_service(service).await.unwrap();
                }
            });
        })
    });
}

fn bench_dependency_resolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("dependency_resolution", |b| {
        b.iter(|| {
            rt.block_on(async {
                let registry = ServiceRegistry::new();
                
                // Create a chain of dependencies: service-0 -> service-1 -> ... -> service-9
                for i in 0..black_box(10) {
                    let mut info = ServiceInfo::new(
                        format!("service-{}", i),
                        "1.0.0",
                        format!("Benchmark service {}", i)
                    );
                    
                    if i > 0 {
                        info = info.with_dependency(format!("service-{}", i - 1));
                    }
                    
                    let service = BaseService::new(info);
                    registry.register_service(service).await.unwrap();
                }
                
                // This triggers dependency resolution
                let _order = registry.get_startup_order().await;
            });
        })
    });
}

fn bench_service_lifecycle(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("service_lifecycle", |b| {
        b.iter(|| {
            rt.block_on(async {
                let registry = ServiceRegistry::new();
                
                // Register services
                for i in 0..black_box(50) {
                    let info = ServiceInfo::new(
                        format!("service-{}", i),
                        "1.0.0",
                        format!("Benchmark service {}", i)
                    );
                    let service = BaseService::new(info);
                    registry.register_service(service).await.unwrap();
                }
                
                // Full lifecycle
                registry.initialize_all().await.unwrap();
                registry.start_all().await.unwrap();
                registry.stop_all().await.unwrap();
            });
        })
    });
}

fn bench_health_checks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let registry = rt.block_on(async {
        let registry = ServiceRegistry::new();
        
        // Register and start services
        for i in 0..100 {
            let info = ServiceInfo::new(
                format!("service-{}", i),
                "1.0.0",
                format!("Benchmark service {}", i)
            );
            let service = BaseService::new(info);
            registry.register_service(service).await.unwrap();
        }
        
        registry.initialize_all().await.unwrap();
        registry.start_all().await.unwrap();
        
        registry
    });
    
    c.bench_function("health_checks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _health = registry.health_check_all().await.unwrap();
            });
        })
    });
}

criterion_group!(
    benches,
    bench_service_registration,
    bench_dependency_resolution,
    bench_service_lifecycle,
    bench_health_checks
);
criterion_main!(benches);