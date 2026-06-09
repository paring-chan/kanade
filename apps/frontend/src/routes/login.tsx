export const Component = () => {
  return (
    <div className="flex justify-center mt-16">
      <div className="max-w-96 w-full">
        <h1 className="text-3xl text-center">로그인</h1>
        <p className="text-center mt-2">로그인에 사용할 서버를 선택하세요</p>
        <div className="flex flex-col gap-2 mt-4">
          <a
            className="border border-black/10 hover:border-black transition-colors px-4 py-2 text-center rounded-lg"
            href="/_/auth/login/aaaaaaaaaaaaaaaaaaa"
          >
            git.pari.ng
          </a>
          <a
            className="border border-black/10 hover:border-black transition-colors px-4 py-2 text-center rounded-lg"
            href="/_/auth/login/aaaaaaaaaaaaaaaaaaa"
          >
            git.pari.ng
          </a>
          <a
            className="border border-black/10 hover:border-black transition-colors px-4 py-2 text-center rounded-lg"
            href="/_/auth/login/aaaaaaaaaaaaaaaaaaa"
          >
            git.pari.ng
          </a>
        </div>
      </div>
    </div>
  );
};
