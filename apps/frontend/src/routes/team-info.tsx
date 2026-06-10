import { Button } from "@base-ui/react";
import { ProjectItem } from "../components/project-item";
import { button } from "../components";

export const Component = () => {
  return (
    <div className="px-4">
      <div className="container mx-auto mt-16">
        <div>팀</div>
        <div className="flex items-end gap-2">
          <h1 className="text-3xl font-bold">미즈키</h1>
          <span className="text-base opacity-60">mizuki</span>
        </div>

        <div className="flex justify-between items-center mt-4">
          <h2 className="text-2xl font-medium">소속 프로젝트</h2>
          <Button className={button({ style: "outlined" })}>생성</Button>
        </div>

        <div className="mt-4 grid lg:grid-cols-2">
          {Array.from({ length: 15 }).map((_, i) => (
            <ProjectItem key={i} />
          ))}
        </div>
      </div>
    </div>
  );
};
