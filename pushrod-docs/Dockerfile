# Serves the prebuilt documentation site (the Docusaurus `build/` folder, including the rustdoc
# API reference in `build/api/`).  The site is built with baseUrl `/`, so it is
# served from `/`.  Built by .github/workflows/docs.yml.
FROM nginx:1.27-alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY build /usr/share/nginx/html

EXPOSE 3003
