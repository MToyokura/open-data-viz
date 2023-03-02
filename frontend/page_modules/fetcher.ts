export const fetcher = (url: string) =>
  fetch(url).then((res) => {
    if (res.status === 404) {
      throw new Error("404 Not Found");
    } else {
      return res.json();
    }
  });
