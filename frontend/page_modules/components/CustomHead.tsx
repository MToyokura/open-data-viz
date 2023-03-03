import Head from "next/head";
import { useRouter } from "next/router";
import Script from "next/script";
import { appName } from "../../pages/_app";

export const CustomHead = (props: { pageTitle?: string }) => {
  const rounter = useRouter();
  return (
    <Head>
      <link
        rel="icon"
        type="image/png"
        href={`${rounter.basePath}/favicon.png`}
      />
      {props.pageTitle ? (
        <title>{`${props.pageTitle} - ${appName}`}</title>
      ) : (
        <title>{appName}</title>
      )}
      <Script
        defer
        data-domain="mtoyokura.github.io"
        src="https://plausible.io/js/script.js"
      />
    </Head>
  );
};
