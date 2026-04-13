<script lang="ts">
  import './layout.css';
  import favicon from '$lib/assets/favicon.png';
  let { children } = $props();

  const siteName = 'S3V';
  const title = 'S3V — S3-Compatible Storage Client';
  const description = 'The easiest way to manage files across multiple S3-compatible buckets. Dual panel, drag & drop, multi-provider support.';
  const url = 'https://s3v.d3fau1t.net';
  const ogImage = `${url}/og-image.png`;
</script>

<svelte:head>
  <!-- Favicon -->
  <link rel="icon" href={favicon} type="image/png" />

  <!-- Primary Meta -->
  <title>{title}</title>
  <meta name="description" content={description} />
  <meta name="author" content="d3fau1t" />
  <meta name="keywords" content="S3, S3V, storage client, bucket manager, AWS S3, MinIO, Cloudflare R2, drag and drop, dual panel, file manager" />
  <link rel="canonical" href={url} />

  <!-- Open Graph -->
  <meta property="og:type" content="website" />
  <meta property="og:url" content={url} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:image" content={ogImage} />
  <meta property="og:site_name" content={siteName} />
  <meta property="og:locale" content="ko_KR" />
  <meta property="og:locale:alternate" content="en_US" />

  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={title} />
  <meta name="twitter:description" content={description} />
  <meta name="twitter:image" content={ogImage} />

  <!-- AI/LLM crawler blocking -->
  <meta name="robots" content="index, follow, noai, noimageai" />
  <meta name="GPTBot" content="noindex, nofollow" />
  <meta name="CCBot" content="noindex, nofollow" />
  <meta name="Google-Extended" content="noindex, nofollow" />
  <meta name="anthropic-index" content="disallow" />
</svelte:head>

<div class="min-h-screen bg-background text-foreground antialiased">
  {@render children()}
</div>
