import { useState } from "react";
import { useMutation } from "@tanstack/react-query";

interface ContextPackage {
  project_id: string;
  original_prompt: string;
  architecture_nodes: any[];
  decisions: any[];
  bugs: any[];
  memories: any[];
  assembled_prompt: string;
  estimated_tokens: number;
}

export function ContextInjection() {
  const [prompt, setPrompt] = useState("");
  const [budget, setBudget] = useState("medium");
  const [result, setResult] = useState<ContextPackage | null>(null);

  const injectMutation = useMutation({
    mutationFn: async () => {
      const res = await fetch("/api/v1/context/inject", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": "Bearer admin-token-123", // Using existing mock token logic
        },
        body: JSON.stringify({
          project_id: "",
          prompt,
          budget,
        }),
      });
      if (!res.ok) throw new Error("Injection failed");
      return res.json();
    },
    onSuccess: (data) => {
      setResult(data);
    },
  });

  return (
    <div className="p-8 max-w-6xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-semibold text-white">Context Injection</h1>
        <div className="px-3 py-1 bg-blue-500/10 text-blue-400 rounded-full text-sm border border-blue-500/20">
          Autonomous Context Engine
        </div>
      </div>

      <div className="bg-[#1A1A1A] p-6 rounded-xl border border-[#333]">
        <h2 className="text-xl text-white mb-4">Generate Prompt Context</h2>
        <div className="space-y-4">
          <div>
            <label className="block text-gray-400 mb-2">User Prompt</label>
            <textarea
              className="w-full bg-[#222] border border-[#333] rounded-lg p-3 text-white h-24"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="e.g., Write a new database repository for users."
            />
          </div>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <label className="block text-gray-400 mb-2">Token Budget</label>
              <select
                className="w-full bg-[#222] border border-[#333] rounded-lg p-3 text-white"
                value={budget}
                onChange={(e) => setBudget(e.target.value)}
              >
                <option value="small">Small (4k tokens)</option>
                <option value="medium">Medium (8k tokens)</option>
                <option value="large">Large (16k tokens)</option>
                <option value="maximum">Maximum (32k tokens)</option>
              </select>
            </div>
            <div className="flex-1 flex items-end h-full">
              <button
                onClick={() => injectMutation.mutate()}
                disabled={injectMutation.isPending || !prompt}
                className="w-full mt-8 bg-blue-600 hover:bg-blue-500 text-white font-medium py-3 px-4 rounded-lg disabled:opacity-50 transition-colors"
              >
                {injectMutation.isPending ? "Injecting Context..." : "Inject Context"}
              </button>
            </div>
          </div>
        </div>
      </div>

      {result && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-6">
            <div className="bg-[#1A1A1A] p-6 rounded-xl border border-[#333]">
              <h2 className="text-lg text-white mb-4">Retrieved Package</h2>
              <div className="grid grid-cols-2 gap-4">
                <div className="bg-[#222] p-4 rounded-lg">
                  <div className="text-gray-400 text-sm">Architecture Nodes</div>
                  <div className="text-2xl text-blue-400">{result.architecture_nodes.length}</div>
                </div>
                <div className="bg-[#222] p-4 rounded-lg">
                  <div className="text-gray-400 text-sm">Decisions</div>
                  <div className="text-2xl text-emerald-400">{result.decisions.length}</div>
                </div>
                <div className="bg-[#222] p-4 rounded-lg">
                  <div className="text-gray-400 text-sm">Bugs</div>
                  <div className="text-2xl text-red-400">{result.bugs.length}</div>
                </div>
                <div className="bg-[#222] p-4 rounded-lg">
                  <div className="text-gray-400 text-sm">Semantic Memories</div>
                  <div className="text-2xl text-purple-400">{result.memories.length}</div>
                </div>
              </div>
            </div>

            <div className="bg-[#1A1A1A] p-6 rounded-xl border border-[#333]">
              <h2 className="text-lg text-white mb-4">Estimated Tokens</h2>
              <div className="text-4xl text-white font-mono">{result.estimated_tokens.toLocaleString()}</div>
            </div>
          </div>

          <div className="bg-[#1A1A1A] p-6 rounded-xl border border-[#333] flex flex-col h-[600px]">
            <h2 className="text-lg text-white mb-4">Final Assembled Prompt</h2>
            <pre className="flex-1 bg-[#222] border border-[#333] rounded-lg p-4 text-gray-300 font-mono text-sm overflow-auto whitespace-pre-wrap">
              {result.assembled_prompt}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}
