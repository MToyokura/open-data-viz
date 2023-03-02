import Head from "next/head";
import Script from "next/script";
import { appName } from "../../pages/_app";

export const CustomHead = (props: { pageTitle?: string }) => {
  return (
    <Head>
      <link rel="icon" type="image/png" href="/favicon.png" />
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
