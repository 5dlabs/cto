Implement subtask 9012: Configure EAS Build profiles and CI environment variables

## Objective
Create eas.json with development, preview, and production build profiles for both iOS and Android. Configure .env.production with EXPO_PUBLIC_API_URL and EXPO_PUBLIC_MORGAN_CHAT_URL. Add EAS build step to CI (GitHub Actions or equivalent) that runs the preview profile on main branch merges. Document how to set EAS secrets for API URLs in the EAS dashboard.

## Steps
eas.json: `{ 'cli': { 'version': '>= 7.0.0' }, 'build': { 'development': { 'developmentClient': true, 'distribution': 'internal', 'env': { 'EXPO_PUBLIC_API_URL': 'https://dev-api.sigma1.com' } }, 'preview': { 'distribution': 'internal', 'android': { 'buildType': 'apk' }, 'env': { 'EXPO_PUBLIC_API_URL': 'https://staging-api.sigma1.com' } }, 'production': { 'autoIncrement': true } } }`. .env.production: `EXPO_PUBLIC_API_URL=https://api.sigma1.com` and `EXPO_PUBLIC_MORGAN_CHAT_URL=https://chat.sigma1.com`. CI step: `- run: npx eas build --platform all --profile preview --non-interactive`. Set EAS_TOKEN as CI secret. Document: `eas secret:create --scope project --name EXPO_PUBLIC_API_URL --value <value>`.

## Validation
Run `eas build --profile preview --platform android --non-interactive` in CI — exits 0, produces downloadable APK artifact. `eas build --profile preview --platform ios --non-interactive` produces IPA (or simulator build). Verify EXPO_PUBLIC_API_URL is accessible in app via `process.env.EXPO_PUBLIC_API_URL` at runtime in preview build.