'use client';

import { useState, useMemo } from 'react';
import { useLiveQuery } from '@tanstack/react-db';
import { settingsCollection, recipesCollection } from '@/db';
import { Button } from '@/components/ui/button';
import { MessageSquare, X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useChat } from '@/lib/ai/use-chat';
import { createOpenAIProvider } from '@/lib/ai/providers/openai';
import { Conversation, ConversationContent, ConversationScrollButton } from './ui/shadcn-io/ai/conversation';
import { Message, MessageAvatar, MessageContent } from './ui/shadcn-io/ai/message';
import { Response } from './ui/shadcn-io/ai/response';
import { PromptInput, PromptInputTextarea, PromptInputToolbar, PromptInputSubmit } from './ui/shadcn-io/ai/prompt-input';
import { type ToolSet } from 'ai';
import { Tool, ToolHeader, ToolContent, ToolInput, ToolOutput } from './ui/shadcn-io/ai/tool';
import { globalTools } from '@/lib/ai-tools/global-tools';
import { createRecipeTools } from '@/lib/ai-tools/recipe-tools';

interface ChatSidebarProps {
  recipeId?: string;
}

export function ChatSidebar({ recipeId }: ChatSidebarProps = {}) {
  const [isOpen, setIsOpen] = useState(false);
  const [input, setInput] = useState('');
  const { data: settingsData } = useLiveQuery(settingsCollection);
  const { data: recipesData } = useLiveQuery(recipesCollection);
  const settings = settingsData?.find((s) => s.id === "user-settings");

  // Find the current recipe if we have a recipeId
  const currentRecipe = recipeId ? recipesData?.find((r) => r.id === recipeId) : undefined;

  // Create the OpenAI model with the API key from settings
  const model = useMemo(() => {
    if (!settings?.openaiApiKey) {
      return null;
    }
    try {
      const openai = createOpenAIProvider(settings.openaiApiKey);
      return openai('gpt-4o-mini');
    } catch (error) {
      console.error('Failed to create OpenAI provider:', error);
      return null;
    }
  }, [settings?.openaiApiKey]);

  // Combine global tools with context-aware recipe tools
  // Recipe tools only available when we're in a recipe context
  const tools: ToolSet = useMemo(() => {
    const recipeContextTools = createRecipeTools(currentRecipe);
    return {
      ...globalTools,
      ...recipeContextTools,
    };
  }, [currentRecipe]);

  // Initial system message changes based on context
  const systemMessage = useMemo(() => {
    if (recipeId) {
      return {
        id: 'system-welcome',
        role: 'assistant' as const,
        parts: [{
          type: 'text' as const,
          text: 'Hi! I can help you with this recipe. I can:\n- View the current recipe (getCurrentRecipe)\n- Modify this recipe by updating ingredients, batch size, etc. (updateCurrentRecipe)\n- Search for ingredients: fermentables, hops, cultures\n- Answer brewing questions\n\nTo modify the recipe, I\'ll first get the current recipe, make your requested changes, then update it.\n\nIMPORTANT: Do not call the same tool multiple times with the same parameters. Complete tasks efficiently in as few steps as possible.'
        }],
      };
    }
    return {
      id: 'system-welcome',
      role: 'assistant' as const,
      parts: [{
        type: 'text' as const,
        text: 'Hi! I can help you with brewing recipes. I can:\n- Create NEW recipes (createRecipe)\n- Search for ingredients: fermentables, hops, cultures\n- List beer styles\n- Answer questions about beer styles and brewing techniques\n\nNavigate to a specific recipe to modify it.\n\nIMPORTANT: Use tools efficiently - avoid calling the same tool repeatedly.'
      }],
    };
  }, [recipeId]);

  const {
    messages,
    sendMessage,
    error,
  } = useChat(model!, tools, {
    messages: [systemMessage],
  });

  return (
    <>
      {/* Toggle Button */}
      {!isOpen && (
        <Button
          className="fixed right-4 bottom-4 z-50 size-14 rounded-full shadow-lg"
          onClick={() => setIsOpen(true)}
          size="icon"
        >
          <MessageSquare className="size-6" />
        </Button>
      )}

      {/* Sidebar */}
      <div
        className={cn(
          "fixed right-0 top-0 z-40 h-full w-96 transform border-l bg-background shadow-2xl transition-transform duration-300",
          isOpen ? "translate-x-0" : "translate-x-full"
        )}
      >
        <div className="flex h-full flex-col">
          {/* Header */}
          <div className="flex flex-col border-b">
            <div className="flex items-center justify-between p-4">
              <div className="flex items-center gap-2">
                <MessageSquare className="size-5" />
                <h2 className="font-semibold">Brewing Assistant</h2>
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setIsOpen(false)}
              >
                <X className="size-5" />
              </Button>
            </div>
            {currentRecipe && (
              <div className="px-4 pb-3 pt-0">
                <div className="text-xs text-muted-foreground bg-muted/50 rounded-md px-2 py-1 inline-block">
                  Recipe: {currentRecipe.recipe.name}
                </div>
              </div>
            )}
          </div>

          {/* Messages */}
          <Conversation>
            <ConversationContent>
              {!model && (
                <div className="text-center text-muted-foreground text-sm p-4">
                  Please add your OpenAI API key in Settings to start chatting.
                </div>
              )}
              {model &&
                messages.map((message, index) => (
                  <Message from={message.role} key={message.id}>
                    <MessageContent>
                      {message.parts?.map((part, i) => {
                        if (part.type === "text") {
                          // Only render if there's actual text content
                          if (!part.text || part.text.trim() === "") {
                            return null;
                          }
                          return (
                            <Response key={`${message.id}-${i}`}>
                              {part.text}
                            </Response>
                          );
                        } else if (part.type.startsWith("tool-")) {
                          const tool = part as any;

                          // Get preview of output
                          let outputPreview = '';
                          if (tool.state === "output-available" && tool.output) {
                            const outputStr = typeof tool.output === 'string'
                              ? tool.output
                              : JSON.stringify(tool.output);

                            // Create a short preview (first 100 chars)
                            outputPreview = outputStr.length > 100
                              ? outputStr.substring(0, 100) + '...'
                              : outputStr;
                          }

                           return (
                             <Tool
                               key={index}
                               defaultOpen={false}
                             >
                               <ToolHeader
                                 type={tool.type}
                                 state={tool.state}
                               />
                               <ToolContent>
                                 <ToolInput input={tool.input} />
                                 {tool.state === "output-available" && (
                                   <div className="text-xs text-muted-foreground mt-1">
                                     {tool.errorText ? (
                                       <div className="text-destructive">Error: {tool.errorText}</div>
                                     ) : outputPreview ? (
                                       <div>Preview: {outputPreview}</div>
                                     ) : null}
                                   </div>
                                 )}
                               </ToolContent>
                             </Tool>
                           );
                        } else {
                          return null;
                        }
                      })}
                    </MessageContent>
                  </Message>
                ))}
              {error && (
                <div className="text-destructive text-sm p-4">
                  Error: {error.message}
                </div>
              )}
            </ConversationContent>
            <ConversationScrollButton />
          </Conversation>

          {/* Input */}
          <div className="border-t p-4">
            <PromptInput
              onSubmit={(e) => {
                e.preventDefault();
                if (input.trim()) {
                  sendMessage({ text: input });
                  setInput(''); // Clear input after sending
                }
              }}
            >
              <PromptInputTextarea
                value={input}
                onChange={(e) => setInput(e.currentTarget.value)}
                placeholder={
                  model
                    ? "Ask about recipes, ingredients, or brewing..."
                    : "Add your OpenAI API key in Settings first"
                }
                disabled={!model}
              />
              <PromptInputToolbar>
                <PromptInputSubmit disabled={!input.trim()} />
              </PromptInputToolbar>
            </PromptInput>
          </div>
        </div>
      </div>

      {/* Backdrop */}
      {isOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/20 backdrop-blur-sm transition-opacity"
          onClick={() => setIsOpen(false)}
        />
      )}
    </>
  );
}
