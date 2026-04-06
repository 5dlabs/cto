Implement subtask 1007: Create sigma1-infra-endpoints ConfigMap aggregating all service endpoints

## Objective
Create the sigma1-infra-endpoints ConfigMap in the databases namespace, aggregating connection strings and service URLs from all provisioned infrastructure resources.

## Steps
1. Create a ConfigMap manifest:
   - name: sigma1-infra-endpoints
   - namespace: databases
   - data:
     POSTGRES_URL: postgresql://user:password@sigma1-pg-rw.databases.svc.cluster.local:5432/app
     POSTGRES_HOST: sigma1-pg-rw.databases.svc.cluster.local
     POSTGRES_PORT: "5432"
     REDIS_URL: redis://:password@sigma1-redis.databases.svc.cluster.local:6379
     REDIS_HOST: sigma1-redis.databases.svc.cluster.local
     REDIS_PORT: "6379"
     S3_ENDPOINT_URL: (from dp-5 decision)
     S3_PRODUCT_IMAGES_BUCKET: sigma1-product-images
     S3_SOCIAL_PHOTOS_BUCKET: sigma1-social-photos
     SIGNALCLI_URL: http://signal-cli.openclaw.svc.cluster.local:8080
     STRIPE_SECRET_REF: sigma1-stripe-credentials
     OPENCORPORATES_SECRET_REF: sigma1-opencorporates-credentials
     LINKEDIN_SECRET_REF: sigma1-linkedin-credentials
     GOOGLE_REVIEWS_SECRET_REF: sigma1-google-reviews-credentials
     INSTAGRAM_SECRET_REF: sigma1-instagram-credentials
     FACEBOOK_SECRET_REF: sigma1-facebook-credentials
2. Note: Actual passwords should be referenced from secrets, not embedded in the ConfigMap. The ConfigMap stores host/port/bucket references; password-bearing URLs are for convenience and should use secretKeyRef in actual pod specs.
3. Apply the ConfigMap.
4. Verify downstream namespaces can read this ConfigMap (via the RBAC set up in 1001).

## Validation
Run `kubectl get configmap sigma1-infra-endpoints -n databases -o yaml` and verify all expected keys are present with non-empty values. From a pod in the sigma1 namespace using envFrom referencing this ConfigMap, verify the env vars are injected correctly.