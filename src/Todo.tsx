import { useLiveQuery } from "@tanstack/react-db";
import { todosCollection } from "./db";
import { useEffect } from "react";

export function Todo() {
  const { data: pinnedNotes } = useLiveQuery(todosCollection);

//   useEffect(() => {
//     todosCollection.insert({ id: "1", text: "1foo", completed: false });
//   }, []);


  return (
    <div>
      {pinnedNotes.map((note) => (
        <div key={note.id}>{note.text}</div>
      ))}
    </div>
  );
}
