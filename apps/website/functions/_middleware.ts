interface Env {
  ASSETS: Fetcher;
}

export const onRequest: PagesFunction<Env> = async (context) => {
  const url = new URL(context.request.url);

  if (!url.hostname.startsWith("cto.")) {
    return context.next();
  }

  const { pathname } = url;

  if (pathname.startsWith("/cto") || pathname.startsWith("/api/")) {
    return context.next();
  }

  if (pathname.startsWith("/_next") || pathname.match(/\.(js|css|png|jpg|jpeg|gif|svg|ico|woff2?|ttf|json|webmanifest|xml|txt)$/)) {
    return context.next();
  }

  url.pathname = `/cto${pathname}`;
  return context.env.ASSETS.fetch(new Request(url.toString(), context.request));
};
