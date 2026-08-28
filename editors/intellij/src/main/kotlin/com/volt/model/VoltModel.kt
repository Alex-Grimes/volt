package com.volt.model

data class FunctionHotspot(
    val name: String,
    val line: Int,
    val end_line: Int,
    val complexity: Int,
    val score: Double
)

data class VoltResult(
    val file_path: String,
    val score: Double,
    val churn: Int,
    val complexity: Int,
    val functions: List<FunctionHotspot>? = null
)

enum class VoltageSeverity {
    HIGH,
    MEDIUM,
    LOW,
    MINIMAL
}

object VoltThresholds {
    var high: Double = 50.0
    var medium: Double = 20.0
    var low: Double = 5.0

    fun getSeverity(score: Double): VoltageSeverity {
        return when {
            score >= high -> VoltageSeverity.HIGH
            score >= medium -> VoltageSeverity.MEDIUM
            score >= low -> VoltageSeverity.LOW
            else -> VoltageSeverity.MINIMAL
        }
    }
}
