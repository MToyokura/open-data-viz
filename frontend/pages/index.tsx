import Link from "next/link";
import { CustomHead } from "../page_modules/components/CustomHead";
import { pageTitles } from "../page_modules/constants";
import { TopPageCard } from "../page_modules/TopPageCard";

export default function Home() {
  return (
    <>
      <CustomHead />
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            padding: "6rem 1rem",
            width: "100%",
            color: "white",
            backgroundColor: "rgb(70, 70, 70)",
          }}
        >
          <h1 style={{ fontSize: "3rem" }}>オープンデータを可視化していく</h1>
          <p>そんなサイトです</p>
        </div>
        <div
          style={{
            background: "rgba(0, 0, 0, 0.03)",
            width: "100%",
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              fontSize: "1.5rem",
              margin: "3rem",
            }}
          >
            可視化したもの一覧
          </div>
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              justifyContent: "center",
              marginBottom: "3rem",
            }}
          >
            <TopPageCard cardHref="/vegetable-production-relay">
              {pageTitles["vegetable-production-relay"]}
            </TopPageCard>
            <TopPageCard cardHref="/workers-by-age-employment-type-education-sex">
              {pageTitles["workers-by-age-employment-type-education-sex"]}
            </TopPageCard>
          </div>
        </div>
        <div
          style={{
            marginTop: "6rem",
          }}
        >
          <Link
            href="/about"
            style={{
              textDecoration: "none",
              color: "grey",
              fontWeight: "bold",
            }}
          >
            {pageTitles.about}
          </Link>
        </div>
      </div>
    </>
  );
}
