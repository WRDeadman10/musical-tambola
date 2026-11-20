import React from "react";
import { Check, RefreshCw } from "lucide-react";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../components/ui/card";
import { MediaValidationReport } from "../../types/media";

interface ValidationReportPanelProps {
    report: MediaValidationReport;
    isValidating: boolean;
    translate: (key: string) => string;
    onValidateAgain: () => void;
    onIssueSelected: (number: number) => void;
}

export function ValidationReportPanel({
    report,
    isValidating,
    translate,
    onValidateAgain,
    onIssueSelected,
}: ValidationReportPanelProps): React.JSX.Element {
    return (
        <Card className="border-indigo-500/40">
            <CardHeader className="flex-row items-center justify-between space-y-0">
                <div>
                    <CardTitle>{translate("media.report.title")}</CardTitle>
                    <p className="mt-1 text-xs text-zinc-400">
                        {report.valid} {translate("media.report.valid")} · {report.invalid}{" "}
                        {translate("media.report.invalid")} · {report.total}{" "}
                        {translate("media.report.total")}
                    </p>
                </div>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={onValidateAgain}
                    disabled={isValidating}
                >
                    <RefreshCw className="h-3.5 w-3.5" />
                    {translate("media.report.validateAgain")}
                </Button>
            </CardHeader>
            <CardContent>
                {report.issues.length === 0 ? (
                    <div className="flex items-center gap-2 text-sm text-emerald-400">
                        <Check className="h-4 w-4" />
                        {translate("media.report.noIssues")}
                    </div>
                ) : (
                    <div className="space-y-2">
                        {report.issues.map((issue) => (
                            <button
                                className="flex w-full items-center gap-3 rounded-md border border-zinc-800 bg-zinc-950/50 p-2 text-left text-sm text-zinc-300 hover:border-indigo-500"
                                key={issue.number}
                                onClick={() => onIssueSelected(issue.number)}
                            >
                                <Badge variant="destructive">{issue.number}</Badge>
                                <span>{issue.message}</span>
                            </button>
                        ))}
                    </div>
                )}
            </CardContent>
        </Card>
    );
}
